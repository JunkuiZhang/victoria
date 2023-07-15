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
    hband_index: u32,
    vband_index: u32,
    band_count: u32,
    width_in_em: f32,
    height_in_em: f32,
};

struct CurveInfo {
    p1: vec2<f32>,
    p2: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> window_size: vec2<f32>;

@group(1) @binding(0)
var<storage, read> font_info: array<GlyphData>;
@group(1) @binding(1)
var<storage, read> font_curves: array<CurveInfo>;
@group(1) @binding(2)
var<storage, read> hband_curves: array<u32>;
@group(1) @binding(3)
var<storage, read> vband_curves: array<u32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.glyph_id = input.glyph_id;
    // Equivlent to:
    // let multiplier_x = input.pixels_per_em / window_size.x * 2.0;
    // let multiplier_y = input.pixels_per_em / window_size.y * 2.0;
    let multiplier = input.pixels_per_em * 2.0 / window_size;
    let this_char_info = font_info[input.glyph_id];
    let scale_x = multiplier.x * this_char_info.width_in_em;
    let scale_y = multiplier.y * this_char_info.height_in_em;
    let scale_mat = mat3x3<f32>(scale_x, 0.0, 0.0, 0.0, scale_y, 0.0, 0.0, 0.0, 1.0);
    let pos_scaled = scale_mat * input.position;
    let move_mat = vec3<f32>(input.base_line.x, input.base_line.y, 0.0);
    out.pos = vec4<f32>(pos_scaled + move_mat, 1.0);
    // Equivlent to the following 2 lines:
    // let transform_mul = vec2<f32>(this_char_info.width_in_em / scale_x, this_char_info.height_in_em / scale_y);
    // out.xy = pos_scaled.xy * transform_mul;
    out.xy = pos_scaled.xy / multiplier;
    out.pixels_per_em = input.pixels_per_em;
    return out;
}

fn band_process(horizontal: bool, pixel: vec2<f32>, pixels_per_em: f32, band_index_start: u32, band_num: u32, curve_index_start: u32) -> f32 {
    var winding_number: f32 = 0.0;
    var x_axis: u32;
    var y_axis: u32;
    if horizontal {
        x_axis = 0u;
        y_axis = 1u;
    } else {
        x_axis = 1u;
        y_axis = 0u;
    }
    var band_offset: u32;
    var total: u32;
    if horizontal {
        band_offset = hband_curves[band_index_start + 2u * band_num];
        total = hband_curves[band_index_start + 2u * band_num + 1u];
    } else {
        band_offset = vband_curves[band_index_start + 2u * band_num];
        total = vband_curves[band_index_start + 2u * band_num + 1u];
    }
    var x: u32 = 0u;
    loop {
        if x >= total {
            break;
        }

        var curve_index_offset: u32;
        if horizontal {
            curve_index_offset = hband_curves[band_index_start + band_offset + x];
        } else {
            curve_index_offset = vband_curves[band_index_start + band_offset + x];
        }
        let curve_index = curve_index_offset + curve_index_start;
        let point0 = font_curves[curve_index - 1u].p2 - pixel;
        let this_curve = font_curves[curve_index];
        let point1 = this_curve.p1 - pixel;
        let point2 = this_curve.p2 - pixel;
        let max_x = max(max(point0[x_axis], point1[x_axis]), point2[x_axis]);
        if max_x * pixels_per_em < -0.5 {
            break;
        }

        var shift_num: u32 = 0u;
        shift_num += u32(step(0.0, point0[y_axis])) * 2u;
        shift_num += u32(step(0.0, point1[y_axis])) * 4u;
        shift_num += u32(step(0.0, point2[y_axis])) * 8u;

        let res = (0x2e74u >> shift_num) & 3u;
        if res == 0u {
            continue;
        }
        // solve the equation: a*t*t - 2*b*t + c = 0
        let a = point0 - 2.0 * point1 + point2;
        let b = point0 - point1;
        let c = point0;
        let d = sqrt(max(b[y_axis] * b[y_axis] - a[y_axis] * c[y_axis], 0.0));
        let ay = 1.0 / a[y_axis];
        var t1: f32;
        var t2: f32;

        if abs(a[y_axis]) < 0.0001 {
            // its a line, not a curve
            t1 = c[y_axis] / (2.0 * b[y_axis]);
            t2 = c[y_axis] / (2.0 * b[y_axis]);
        } else {
            t1 = (b[y_axis] - d) * ay;
            t2 = (b[y_axis] + d) * ay;
        }

        if (res & 0x01u) > 0u {
            let x1 = (a[x_axis] * t1 - 2.0 * b[x_axis]) * t1 + c[x_axis];
            winding_number = winding_number + clamp(pixels_per_em * x1 + 0.5, 0.0, 1.0);
        }

        if res > 1u {
            let x2 = (a[x_axis] * t2 - 2.0 * b[x_axis]) * t2 + c[x_axis];
            winding_number = winding_number - clamp(pixels_per_em * x2 + 0.5, 0.0, 1.0);
        }

        continuing {
            x = x + 1u;
        }
    }

    return winding_number;
}

@fragment
fn fs_main(input: FragmengInput) -> @location(0) vec4<f32> {
    let glyph_data = font_info[input.glyph_id];
    if glyph_data.width_in_em < 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // transform to em coordinate system
    var winding_number: f32 = 0.0;
    let temp_color: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let hband_num = u32(input.position.y / glyph_data.height_in_em * f32(glyph_data.band_count));
    let vband_num = u32(input.position.x / glyph_data.width_in_em * f32(glyph_data.band_count));

    winding_number = winding_number + band_process(true, input.position, input.pixels_per_em, glyph_data.hband_index, hband_num, glyph_data.curve_texel_index);
    winding_number = winding_number + abs(band_process(false, input.position, input.pixels_per_em, glyph_data.vband_index, vband_num, glyph_data.curve_texel_index));

    // Take the average of the horizontal and vertical results. The absolute
	// value ensures that either winding convention works. The square root
	// approximates gamma correction.
    winding_number = sqrt(clamp(winding_number * 0.5, 0.0, 1.0));

    if winding_number > 0.0001 {
        return temp_color * winding_number;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
        // return vec4<f32>(0.5, 0.7, 0.2, 1.0);
    }
    // return vec4<f32>(1.0, 0.7, 0.5, 1.0);
}
