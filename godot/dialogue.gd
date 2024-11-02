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
	$TextArea.set_visible(false)
	

func _input(event):
	if event.is_action_pressed("mouse_click") and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		if ongoing_text:
			advance_text()
		else:
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
