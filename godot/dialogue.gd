extends Control

@onready var history = $History

@onready var textbox = get_node("TextArea/TextBox")
@onready var text = [ "Welcome to my shop!",  "Feel free to look around."]
@onready var i = 0

func _ready():
	history.generate_history()
	var item = history.get_item(0)
	print(item.item_type)
	print(item.description)
	for story in item.stories:
		print(story.lines)
	
	textbox.text = text[0]

func _input(event):
	if event.is_action_pressed("mouse_click") and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		var item = history.get_item(0)
		i += 1
		if i >= len(item.stories[-1].lines):
			i = 0
		textbox.text = item.stories[-1].lines[i]
