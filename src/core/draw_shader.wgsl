struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) xy: vec2<f32>,
};

struct CurveInfo {
    max: vec2<f32>,
    p0: vec2<f32>,
    p1: vec2<f32>,
    p2: vec2<f32>,
};

@group(0) @binding(0)
var<storage, read> x_curve_list: array<CurveInfo>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(input.position, 1.0);
    out.xy = vec2<f32>(input.position.x, input.position.y);
    return out;
}

@fragment
fn fs_main(@location(0) input: vec2<f32>) -> @location(0) vec4<f32> {
    let total = arrayLength(&x_curve_list);
    let pixel_per_em = 768.0 * 0.7;
    // transform to em coordinate system
    let pixel = vec2<f32>(input.x + 0.7, input.y + 0.7) / 1.4;

    var winding_number: f32 = 0.0;
    var temp_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    temp_color = vec3<f32>(1.0, 1.0, 1.0);

    var x: u32 = 0u;
    loop {
        // winding_number = winding_number + 0.5;
        if x >= total {
            break;
        }

        let max = x_curve_list[x].max.x - pixel.x;
        if max * pixel_per_em < -0.5 {
            break;
        }

        let point0 = x_curve_list[x].p0 - pixel;
        let point1 = x_curve_list[x].p1 - pixel;
        let point2 = x_curve_list[x].p2 - pixel;
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
        let res = 0x2e74 >> shift_num;
        if (res & 0x01) > 0 {
            let a = point0.y - 2.0 * point1.y + point2.y;
            let b = point0.y - point1.y;
            let c = point0.y;
            let t = (b - sqrt(b * b - a * c)) / a;
            let x = (point0.x - 2.0 * point1.x + point2.x) * t * t - 2.0 * (point0.x - point1.x) * t + point0.x;
            if x >= 0.0 {
                // winding_number = winding_number + clamp(pixel_per_em * x + 0.5, 0.0, 1.0);
                winding_number = winding_number + 1.0;
            }
        }
        if (res & 0x02) > 0 {
            let a = point0.y - 2.0 * point1.y + point2.y;
            let b = point0.y - point1.y;
            let c = point0.y;
            let t = (b + sqrt(b * b - a * c)) / a;
            let x = (point0.x - 2.0 * point1.x + point2.x) * t * t - 2.0 * (point0.x - point1.x) * t + point0.x;
            if x >= 0.0 {
                // winding_number = winding_number - clamp(pixel_per_em * x + 0.5, 0.0, 1.0);
                winding_number = winding_number - 1.0;
            }
        }
        // winding_number = winding_number + 0.07;

        continuing {
            x = x + 1u;
        }
    }
    // winding_number = 0.7;
    // if pixel.x > 0.0 {
    //     temp_color.x = 1.0;
    //     if pixel.x >= 1.0 {
    //         temp_color.x = 0.5;
    //     }
    // }
    // if pixel.y > 0.0 {
    //     temp_color.y = 1.0;
    //     if pixel.y >= 1.0 {
    //         temp_color.y = 0.5;
    //     }
    // }

    // return vec4<f32>(1.0, 1.0, 1.0, winding_number);
    return vec4<f32>(temp_color, winding_number);
}
