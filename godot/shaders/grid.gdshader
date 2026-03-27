shader_type spatial;
render_mode unshaded;

uniform float line_width: hint_range(0.01, 0.05) = 0.02;

float pristine_grid(in vec2 uv) {
    vec2 line_w = vec2(line_width);
    vec2 ddx = dFdx(uv);
    vec2 ddy = dFdy(uv);
    vec2 uvDeriv = vec2(length(vec2(ddx.x, ddy.x)), length(vec2(ddx.y, ddy.y)));
    bvec2 invertLine = bvec2(line_w.x > 0.5, line_w.y > 0.5);
    vec2 targetWidth = vec2(
      invertLine.x ? 1.0 - line_w.x : line_w.x,
      invertLine.y ? 1.0 - line_w.y : line_w.y
      );
    vec2 drawWidth = clamp(targetWidth, uvDeriv, vec2(0.5));
    vec2 lineAA = uvDeriv * 1.5;
    vec2 gridUV = abs(fract(uv) * 2.0 - 1.0);
    gridUV.x = invertLine.x ? gridUV.x : 1.0 - gridUV.x;
    gridUV.y = invertLine.y ? gridUV.y : 1.0 - gridUV.y;
    vec2 grid2 = smoothstep(drawWidth + lineAA, drawWidth - lineAA, gridUV);

    grid2 *= clamp(targetWidth / drawWidth, 0.0, 1.0);
    grid2 = mix(grid2, targetWidth, clamp(uvDeriv * 2.0 - 1.0, 0.0, 1.0));
    grid2.x = invertLine.x ? 1.0 - grid2.x : grid2.x;
    grid2.y = invertLine.y ? 1.0 - grid2.y : grid2.y;
    return mix(grid2.x, 1.0, grid2.y);
}

float sdCircle(vec2 p, float r) {
    return length(p) - r;
}

void fragment() {
    vec2 uv = (INV_VIEW_MATRIX * vec4(VERTEX, 1.0)).xz;
    vec3 color = vec3(pristine_grid(uv));
    float fade = clamp(-sdCircle(UV - 0.5, 0.45), 0.0, 1.0);

    ALBEDO = color;
    ALPHA = color.x * fade;
}
