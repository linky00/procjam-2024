extends Control

@onready var history = $History

@onready var textbox = get_node("TextArea/Margins/TextBox")
@onready var text = []
@onready var i = 0
@onready var ongoing_text = false

signal loadResource(item:String, texture:Resource)

@onready var sprites = {
	"vase1": load("res://sprites/vase1.png"),
	"vase2": load("res://sprites/vase2.png"),
	"vase3": load("res://sprites/vase3.png")
}

func _ready():
	history.generate_history()
		
	for i in range(3):
		print(i)
		var item = history.get_item(i)
		print(item.item_type)
		print(item.description)
		for story in item.stories:
			print(story.lines)
	

func _input(event):
	if event.is_action_pressed("mouse_click") and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		if ongoing_text:
			advance_text()
		else:
			print(history)
			show_text(["Welcome to my shop!",  "Feel free to look around."])

func show_text(new_text: Array):
	text = new_text
	i = 0
	ongoing_text = true
	textbox.text = text[0]
	$TextArea.set_visible(true)

func advance_text():
	i += 1
	if i < len(text):
		textbox.text = text[i]
	else:
		$TextArea.set_visible(false)
		ongoing_text = false
	
func on_item_approach(item: String):
	show_text(["Ah, I see you're interested in item " + item + "..."])
