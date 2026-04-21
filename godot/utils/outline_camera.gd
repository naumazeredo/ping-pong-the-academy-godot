extends Camera3D

@export var main_camera: Camera3D

func _process(_delta: float) -> void:
	global_transform = main_camera.global_transform
