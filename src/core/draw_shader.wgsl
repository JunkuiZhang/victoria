struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) glyph_id: u32,
    @location(2) base_line: vec2<f32>,
    @location(3) pixels_per_em: f32,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) xy: vec2<f32>,
    @location(1) pixels_per_em: f32,
    @location(2) glyph_id: u32,
};

struct FragmengInput {
    @location(0) position: vec2<f32>,
    @location(1) pixels_per_em: f32,
    @location(2) glyph_id: u32,
};

struct GlyphData {
    curve_texel_index: u32,
    curve_info_index: u32,
    width_over_height: f32,
    width_in_em: f32,
};

struct CurveInfo {
    p1: vec2<f32>,
    p2: vec2<f32>,
};

@group(0) @binding(0)
var<storage, read> font_info: array<GlyphData>;
@group(0) @binding(1)
var<storage, read> font_curves: array<CurveInfo>;
@group(0) @binding(2)
var<storage, read> curve_orders: array<u32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.glyph_id = input.glyph_id;
    // column left to right
    let multiplier = input.pixels_per_em / 768.0 * 2.0;
    let scale_x = font_info[input.glyph_id].width_over_height * multiplier;
    let scale_y = multiplier;
    let scale_mat = mat3x3<f32>(scale_x, 0.0, 0.0, 0.0, scale_y, 0.0, 0.0, 0.0, 1.0);
    let pos_scaled = scale_mat * input.position;
    let move_mat = vec3<f32>(input.base_line.x, input.base_line.y, 0.0);
    out.pos = vec4<f32>(pos_scaled + move_mat, 1.0);
    let transform_mul = font_info[input.glyph_id].width_in_em / scale_x;
    out.xy = pos_scaled.xy * transform_mul;
    // out.xy = pos_scaled.xy / vec2<f32>(scale_x, scale_y);
    out.pixels_per_em = input.pixels_per_em;
    return out;
}

@fragment
fn fs_main(input: FragmengInput) -> @location(0) vec4<f32> {
    var indicator: bool = false; // TODO: Delete
    let epsilon: f32 = 0.0001;
    let glyph_id = input.glyph_id;
    let glyph_data = font_info[glyph_id];
    if glyph_data.width_over_height < 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let total = curve_orders[glyph_data.curve_texel_index];
    // transform to em coordinate system
    let pixel = input.position;

    var winding_number: f32 = 0.0;
    var temp_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    temp_color = vec3<f32>(1.0, 1.0, 1.0);

    var x: u32 = 0u;
    loop {
        if x >= total {
            break;
        }
        let curve_index_data = curve_orders[glyph_data.curve_info_index + x + 1u];
        let curve_index_row_num = curve_index_data >> 16u;
        let curve_index_col_num = curve_index_data & 0xFFFFu;
        let curve_index = curve_index_col_num + curve_index_row_num * 4096u;
        let point0 = font_curves[curve_index - 1u].p2 - pixel;
        let origin_data = font_curves[curve_index - 1u].p2;
        if origin_data.x < 0.0 || origin_data.y < 0.0 {
            indicator = true;
        }
        let this_curve = font_curves[curve_index];
        let point1 = this_curve.p1 - pixel;
        let point2 = this_curve.p2 - pixel;
        let max_x = max(max(point0.x, point1.x), point2.x);
        if max_x * input.pixels_per_em < -0.5 {
            break;
        }

        var shift_num: u32 = 0u;
        if point0.y > 0.0 {
            shift_num = shift_num + 2u;
        }
        if point1.y > 0.0 {
            shift_num = shift_num + 4u;
        }
        if point2.y > 0.0 {
            shift_num = shift_num + 8u;
        }

        let res = (0x2e74u >> shift_num) & 3u;
        if res == 0u {
            continue;
        }
        // solve the equation: a*t*t - 2*b*t + c = 0
        let a = point0 - 2.0 * point1 + point2;
        let b = point0 - point1;
        let c = point0;
        let d = sqrt(max(b.y * b.y - a.y * c.y, 0.0));
        let ay = 1.0 / a.y;
        var t1: f32;
        var t2: f32;

        if abs(a.y) < epsilon {
            // its a line, not a curve
            t1 = c.y / (2.0 * b.y);
            t2 = c.y / (2.0 * b.y);
        } else {
            t1 = (b.y - d) * ay;
            t2 = (b.y + d) * ay;
        }

        if (res & 0x01u) > 0u {
            let x1 = (a.x * t1 - 2.0 * b.x) * t1 + c.x;
            winding_number = winding_number + clamp(input.pixels_per_em * x1 + 0.5, 0.0, 1.0);
        }

        if res > 1u {
            let x2 = (a.x * t2 - 2.0 * b.x) * t2 + c.x;
            winding_number = winding_number - clamp(input.pixels_per_em * x2 + 0.5, 0.0, 1.0);
        }

        continuing {
            x = x + 1u;
        }
    }

    if winding_number > epsilon {
        return vec4<f32>(temp_color, winding_number);
    } else {
        let float_number = vec2<f32>(1.05, 1.7);
        let my_number = vec2<u32>(float_number);
        if indicator {
            return vec4<f32>(0.7, 0.2, 0.2, 1.0);
        } else {
            return vec4<f32>(0.2, 0.7, 0.2, 1.0);
        }
    }
    // return vec4<f32>(1.0, 0.7, 0.5, 1.0);
}
