shader_type spatial;
render_mode unshaded, blend_mix, depth_draw_never, depth_test_disabled;

uniform sampler2D my_texture : source_color;

void vertex() {
	POSITION = vec4(VERTEX.xy, 1.0, 1.0);
}

void fragment() {
	vec4 color = texture(my_texture, SCREEN_UV);
	ALBEDO = color.rgb;
	ALPHA = color.a;
}
