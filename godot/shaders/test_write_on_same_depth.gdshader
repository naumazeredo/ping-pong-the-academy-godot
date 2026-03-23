shader_type spatial;
render_mode depth_test_disabled;

uniform sampler2D depth_texture : source_color, hint_depth_texture;

void fragment() {
	float depth_raw = texture(depth_texture, SCREEN_UV).x;
	float fragment_depth = FRAGCOORD.z;

	float EPS = 0.001;
	if (fragment_depth > depth_raw - EPS && fragment_depth < depth_raw + EPS) {
		ALBEDO = vec3(1.0, 0.0, 0.0);
		ALPHA = 0.5;
	} else {
		discard;
	}
}
