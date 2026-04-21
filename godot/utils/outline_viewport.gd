extends SubViewport


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	self.size = get_tree().root.size

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta: float) -> void:
	self.size = get_tree().root.size
