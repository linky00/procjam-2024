extends Area3D

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass

func _on_dialogue_load_resource(item: String, texture: Resource) -> void:
	if name == item:
		$Sprite.texture = texture
