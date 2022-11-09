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
    let epsilon: f32 = 0.0001;
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

        let max_x = x_curve_list[x].max.x - pixel.x;
        if max_x * pixel_per_em < -0.5 {
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
            winding_number = winding_number + clamp(pixel_per_em * x1 + 0.5, 0.0, 1.0);
            // winding_number = winding_number + 1.0;
        }

        if res > 1u {
            let x2 = (a.x * t2 - 2.0 * b.x) * t2 + c.x;
            winding_number = winding_number - clamp(pixel_per_em * x2 + 0.5, 0.0, 1.0);
            // winding_number = winding_number - 1.0;
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
