shader_type spatial;
render_mode depth_draw_always;

void fragment() {
	DEPTH = FRAGCOORD.z;
	ALPHA = 0.0;
}
