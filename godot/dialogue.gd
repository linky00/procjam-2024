extends Control

@onready var history = $History

func _ready():
	history.generate_history()
	var item = history.get_item(0)
	print(item.item_type)
	print(item.description)
	for story in item.stories:
		print(story.lines)
