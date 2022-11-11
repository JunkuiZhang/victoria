struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) xy: vec2<f32>,
};

struct FontRect {
    pixels_per_em: f32,
    units_per_em: f32,
    pixels: vec2<f32>,
    coordinate_in_vertx: vec2<f32>,
    coordinate_in_units: vec2<f32>,
};

struct CurveInfo {
    max: vec2<f32>,
    p0: vec2<f32>,
    p1: vec2<f32>,
    p2: vec2<f32>,
};

@group(0) @binding(0)
var<storage, read> font_info: FontRect;
@group(0) @binding(1)
var<storage, read> font_curves: array<CurveInfo>;

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
    let total = arrayLength(&font_curves);
    // transform to em coordinate system
    let pixel = (input - font_info.coordinate_in_vertx) * font_info.pixels / font_info.pixels_per_em + (font_info.coordinate_in_units / font_info.units_per_em);

    var winding_number: f32 = 0.0;
    var temp_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    temp_color = vec3<f32>(1.0, 1.0, 1.0);

    var x: u32 = 0u;
    loop {
        if x >= total {
            break;
        }

        let max_x = font_curves[x].max.x - pixel.x;
        if max_x * font_info.pixels_per_em < -0.5 {
            break;
        }

        let point0 = font_curves[x].p0 - pixel;
        let point1 = font_curves[x].p1 - pixel;
        let point2 = font_curves[x].p2 - pixel;
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
            winding_number = winding_number + clamp(font_info.pixels_per_em * x1 + 0.5, 0.0, 1.0);
        }

        if res > 1u {
            let x2 = (a.x * t2 - 2.0 * b.x) * t2 + c.x;
            winding_number = winding_number - clamp(font_info.pixels_per_em * x2 + 0.5, 0.0, 1.0);
        }

        continuing {
            x = x + 1u;
        }
    }

    if winding_number > epsilon {
        return vec4<f32>(temp_color, winding_number);
    } else {
        let my_number = vec2<u32>(u32(1.05), u32(1.7));
        if (my_number.x == 1u) && (my_number.y == 1u) {
            return vec4<f32>(0.2, 0.7, 0.2, 1.0);
        } else {
            return vec4<f32>(0.7, 0.2, 0.2, 1.0);
        }
    }
    // return vec4<f32>(1.0, 0.7, 0.5, 1.0);
}
