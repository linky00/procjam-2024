extends Control

@onready var history = $History

@onready var textbox = get_node("TextArea/Margins/TextBox")
@onready var text = []
@onready var i = 0
@onready var ongoing_text = false

signal loadResource(item:String, texture:Resource)
signal changeShopkeeperSprite(texture:Resource)

@onready var sprites = {
	"vase1": load("res://sprites/vase1.png"),
	"vase2": load("res://sprites/vase2.png"),
	"vase3": load("res://sprites/vase3.png"),
	"teapot1": load("res://sprites/teapot1.png"),
	"teapot2": load("res://sprites/teapot2.png"),
	"teapot3": load("res://sprites/teapot3.png"),
	"belt1": load("res://sprites/belt.png"),
	"bracelet1": load("res://sprites/bracelet.png"),
	"cup1": load("res://sprites/cup1.png"),
	"hat1": load("res://sprites/hat.png"),
	"orb1": load("res://sprites/orb.png"),
	"shoes1": load("res://sprites/shoes1.png"),
	"shoes2": load("res://sprites/shoes2.png"),
	"shoes3": load("res://sprites/shoes3.png"),
	"statue1": load("res://sprites/statue.png"),
	"sunglasses1": load("res://sprites/sunglasses.png"),
	"necklace1" :load("res://sprites/necklace.png")
	
}

@onready var shopkeeper_sprites = {
	"standing": load("res://sprites/shopkeeper_1.png"),
	"hand_up": load("res://sprites/shopkeeper_2.png")
}

var story_counts = {
	"Item1": 0,
	"Item2": 0,
	"Item3": 0,
	"Item4": 0,
	"Item5": 0,
	"Item6": 0,
	"Default": 0
}

var last_item = null

var default_stories = [
	[
		"Welcome to my shop! Feel free to have a look around."
	], [
		"If you see anything that catches your eye, please don't hesitate to come speak to me.",
		"I can't promise I remember everything about all of these items, but I'm always happy to give it my best shot."
	]
]

func _ready():
	
		
	$TextArea.set_visible(false)
	history.generate_history()
	for i in range(6):
		var item = history.get_item(i)
		emit_signal("loadResource", "Item"+str(i+1), sprites[item.item_type])

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
		emit_signal("changeShopkeeperSprite", shopkeeper_sprites["standing"])
	
func _on_player_approach(item_name: String):
	if ongoing_text:
		advance_text()	
	else:
		if item_name == "Shopkeeper":
			emit_signal("changeShopkeeperSprite", shopkeeper_sprites["hand_up"])
			if last_item:
				show_item_stories(last_item)
			else:
				show_text(default_stories[story_counts["Default"]])
				
				story_counts["Default"] += 1
				if story_counts["Default"] >= len(default_stories):
					story_counts["Default"] = 0
		else:
			var item_num = int(item_name.substr(4)) - 1
			var item = history.get_item(item_num)
			
			show_text(item.description + ["Maybe you could ask the shopkeeper about this item..."])
			last_item = item_name


func show_item_stories(item_name: String):
	var item_num = int(item_name.substr(4)) - 1
	var item = history.get_item(item_num)
	
	var i = item.stories.size() - 1 - story_counts[item_name]
	print(i)
	print(item.stories[i].lines)
	
	if story_counts[item_name] == 0:
		show_text(["I see you're interested in the "+item_name+",,,"]+item.stories[i].lines)
	else:
		show_text(item.stories[i].lines)
	
	story_counts[item_name] += 1
	if story_counts[item_name] >= len(item.stories):
		story_counts[item_name] = 0

func _on_player_advance() -> void:
	if ongoing_text:
		advance_text()
