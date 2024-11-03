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
	"vase3": load("res://sprites/vase3.png"),
	"teapot1": load("res://sprites/teapot1.png"),
	"teapot2": load("res://sprites/teapot2.png"),
	"teapot3": load("res://sprites/teapot3.png")
}

var story_counts = {
	"Item1": 0,
	"Item2": 0,
	"Item3": 0,
	"Default": 0
}

var default_stories = [
	[
		"Welcome to my shop! Feel free to have a look around."
	], [
		"If you see anything that catches your eye, please don't hesitate to come speak to me.",
		"I can't promise I remember everything about all of these items, but I'm always happy to give it my best shot."
	]
]

func _ready():
	history.generate_history()
		
	$TextArea.set_visible(false)
	for i in range(3):
		var item = history.get_item(i)
		emit_signal("loadResource", "Item"+str(i+1), sprites[item.item_type])
#		print(i)

#		print(item.item_type)
#		print(item.description)
#		for story in item.stories:
#			print(story.lines)
#		print("Item"+str(i+1))

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
	
func _on_player_approach(item_name: String):
	if not ongoing_text:
		var item_num = int(item_name.substr(4)) - 1
		var item = history.get_item(item_num)
		
		var i = item.stories.size() - 1 - story_counts[item_name]
		print(i)
		print(item.stories[i].lines)
		
		show_text(item.stories[i].lines)
		
		story_counts[item_name] += 1
		if story_counts[item_name] >= len(item.stories):
			story_counts[item_name] = 0
		
	else:
		advance_text()


func _on_player_advance() -> void:
	if ongoing_text:
		advance_text()
	else:
		if story_counts["Default"] < len(default_stories):
			show_text(default_stories[story_counts["Default"]])
			story_counts["Default"] += 1
