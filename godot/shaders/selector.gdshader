shader_type spatial;
render_mode unshaded;

uniform vec2 size = vec2(1.0, 1.0);
uniform float border_size : hint_range(0, 1) = 0.2;
uniform float corner_size : hint_range(0, 1) = 0.5;
uniform float inner_roundness : hint_range(0, 1) = 0.0;
uniform float outer_roundness : hint_range(0, 1) = 0.0;
uniform float margin : hint_range(0, 1) = 0.2;

float sdRoundedBox(in vec2 p, in vec2 b, in vec4 r) {
	r.xy = (p.x > 0.0) ? r.xy : r.zw;
	r.x = (p.y > 0.0) ? r.x : r.y;
	vec2 q = abs(p) - b + r.x;
	return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
}

float sdBox(in vec2 p, in vec2 b) {
	vec2 d = abs(p) - b;
	return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
}

float sdCorner(in vec2 p, in float c) {
	vec2 d = abs(p) - 0.5 * size + c;
	return min(d.x, d.y);
}

void fragment() {
	// XXX: this way of working with SDFs doesn't work for the margin effect. Internal values doesn't work properly.

	vec2 half_size = size * 0.5;
	vec2 uv = UV * size - half_size;

	float inner_box_sdf = sdBox(uv, half_size - margin - border_size - inner_roundness);
	float inner_border_sdf = inner_box_sdf - inner_roundness;

	float outer_box_sdf = sdBox(uv, half_size - margin - outer_roundness);
	float outer_border_sdf = outer_box_sdf - outer_roundness;

	float border_sdf = min(inner_border_sdf, -outer_border_sdf);
	float corner_sdf = sdCorner(uv, corner_size + margin);

	float border_corner_sdf = min(border_sdf, corner_sdf);

	float val = (border_corner_sdf > 0.0) ? 1.0 : 0.0;
	ALBEDO = vec3(val);
	ALPHA = val;
}
