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

@fragment
fn fs_main(input: FragmengInput) -> @location(0) vec4<f32> {
    let epsilon: f32 = 0.0001;
    let glyph_id = input.glyph_id;
    let glyph_data = font_info[glyph_id];
    if glyph_data.width_in_em < 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // transform to em coordinate system
    let pixel = input.position;
    var winding_number: f32 = 0.0;
    let temp_color: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let band_count = glyph_data.band_count;

        {// hband
        let hband_index = u32(pixel.y / glyph_data.height_in_em * f32(band_count));
        let hband_start = hband_curves[glyph_data.hband_index + 2u * hband_index];
        let total = hband_curves[glyph_data.hband_index + 2u * hband_index + 1u];
        var x: u32 = 0u;
        loop {
            if x >= total {
            break;
            }

            // let curve_index_offset = hband_curves[hband_start + x];
            let curve_index_offset = hband_curves[glyph_data.hband_index + hband_start + x];
            let curve_index = curve_index_offset + glyph_data.curve_texel_index;
            let point0 = font_curves[curve_index - 1u].p2 - pixel;
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
    }

    //     {// vband
    //     let vband_index = u32(pixel.x / glyph_data.width_in_em * f32(band_count));
    //     let vband_start = vband_curves[glyph_data.hband_index + 2u * vband_index];
    //     let total = vband_curves[glyph_data.hband_index + 2u * vband_index + 1u];
    //     var x: u32 = 0u;
    //     loop {
    //         if x >= total {
    //         break;
    //         }

    //         let curve_index_offset = vband_curves[glyph_data.vband_index + vband_start + x];
    //         let curve_index = curve_index_offset + glyph_data.curve_texel_index;
    //         let point0 = font_curves[curve_index - 1u].p2 - pixel;
    //         let origin_data = font_curves[curve_index - 1u].p2;
    //         let this_curve = font_curves[curve_index];
    //         let point1 = this_curve.p1 - pixel;
    //         let point2 = this_curve.p2 - pixel;
    //         let max_x = max(max(point0.y, point1.y), point2.y);
    //         if max_x * input.pixels_per_em < -0.5 {
    //         break;
    //         }

    //         var shift_num: u32 = 0u;
    //         if point0.x > 0.0 {
    //             shift_num = shift_num + 2u;
    //         }
    //         if point1.x > 0.0 {
    //             shift_num = shift_num + 4u;
    //         }
    //         if point2.x > 0.0 {
    //             shift_num = shift_num + 8u;
    //         }

    //         let res = (0x2e74u >> shift_num) & 3u;
    //         if res == 0u {
    //         continue;
    //         }
    //     // solve the equation: a*t*t - 2*b*t + c = 0
    //         let a = point0 - 2.0 * point1 + point2;
    //         let b = point0 - point1;
    //         let c = point0;
    //         let d = sqrt(max(b.x * b.x - a.x * c.x, 0.0));
    //         let ay = 1.0 / a.x;
    //         var t1: f32;
    //         var t2: f32;

    //         if abs(a.x) < epsilon {
    //         // its a line, not a curve
    //             t1 = c.x / (2.0 * b.x);
    //             t2 = c.x / (2.0 * b.x);
    //         } else {
    //             t1 = (b.x - d) * ay;
    //             t2 = (b.x + d) * ay;
    //         }

    //         if (res & 0x01u) > 0u {
    //             let x1 = (a.y * t1 - 2.0 * b.y) * t1 + c.y;
    //             winding_number = winding_number + clamp(input.pixels_per_em * x1 + 0.5, 0.0, 1.0);
    //         }

    //         if res > 1u {
    //             let x2 = (a.x * t2 - 2.0 * b.x) * t2 + c.x;
    //             winding_number = winding_number - clamp(input.pixels_per_em * x2 + 0.5, 0.0, 1.0);
    //         }

    //         continuing {
    //             x = x + 1u;
    //         }
    //     }
    // }

    // Take the average of the horizontal and vertical results. The absolute
	// value ensures that either winding convention works. The square root
	// approximates gamma correction.
    // winding_number = sqrt(clamp(abs(winding_number) * 0.5, 0.0, 1.0));

    if winding_number > epsilon {
        return temp_color * winding_number;
    } else {
        // return vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return vec4<f32>(0.5, 0.7, 0.2, 1.0);
    }
    // return vec4<f32>(1.0, 0.7, 0.5, 1.0);
}
