PROMPT:

I am a dungeon master. I need to create a setup for the game.

I need a list of each room. 
    In each room I need a list of monsters. 
    In each room I need a list of items.
I need a list of room connections.
I need a list of monsters with any special abilities they might have.

Each room should be described with the following structure:

    Room number: The room number.
    Name: The name of the room.
    Description: The description of the room.
    Monsters: A comma separated list of monsters in the room.
    Items: A comma separated list of items in the room.

Each room connection should be described with the following structure:

    Room Connections: A list of connected room numbers.

Each monstor should be described with the following structure:

    Monstor name: The name of a monster.
    Description: A short description of the monster.
    Abilities: A comma separated list of special abilties the monster has.


I need the response in JSON format. "rooms" should be an array with the following structure:
{
    "roomNumber": {The room number},
    "name": {The name of the room},
    "description": {The description of the room},
    "monsters": [A comma separated list of monsters in the room],
    "items": [A comma separated list of items in the room]
}

"monsters" should be an array with the following structure:
{
    "name": {The name of a monster},
    "description": {A short description of the monster},
    "abilities": [A comma separated list of special abilties the monster has]
}

"roomConnections" should be an arry with the following structure:
{
    "roomNumber": 1,
    "connectedRooms": [A list of connected room numbers]
}



RESPONSE (ChatGPT):

{
  "rooms": [
    {
      "roomNumber": 1,
      "name": "The Dim Entrance",
      "description": "A dark, dusty entrance that opens into a gloomy dungeon.",
      "monsters": ["Skeleton Warrior", "Bat"],
      "items": ["Ancient Key", "Torch"]
    },
    {
      "roomNumber": 2,
      "name": "The Hall of Mirrors",
      "description": "A long hallway lined with broken mirrors and filled with eerie reflections.",
      "monsters": ["Phantom", "Mirror Demon"],
      "items": ["Mirror Shard", "Health Potion"]
    },
    {
      "roomNumber": 3,
      "name": "The Hidden Library",
      "description": "A secret library filled with dusty books and arcane knowledge.",
      "monsters": ["Animated Book", "Book Wyrm"],
      "items": ["Spellbook", "Scroll of Wisdom"]
    },
    {
      "roomNumber": 4,
      "name": "The Forgotten Crypt",
      "description": "A dark crypt where ancient undead roam among the sarcophagi.",
      "monsters": ["Zombie", "Skeleton Mage"],
      "items": ["Cursed Amulet", "Mummified Hand"]
    },
    {
      "roomNumber": 5,
      "name": "The Enchanted Garden",
      "description": "A magical garden with glowing plants and a mystical atmosphere.",
      "monsters": ["Pixie", "Treant"],
      "items": ["Enchanted Flower", "Magic Seeds"]
    }
  ],
  "roomConnections": [
    {
      "roomNumber": 1,
      "connectedRooms": [2, 3]
    },
    {
      "roomNumber": 2,
      "connectedRooms": [1, 4]
    },
    {
      "roomNumber": 3,
      "connectedRooms": [1]
    },
    {
      "roomNumber": 4,
      "connectedRooms": [2, 5]
    },
    {
      "roomNumber": 5,
      "connectedRooms": [4]
    }
  ],
  "monsters": [
    {
      "name": "Skeleton Warrior",
      "description": "A reanimated skeleton armed with a rusty sword.",
      "abilities": ["Immunity to poison", "Basic melee attacks"]
    },
    {
      "name": "Bat",
      "description": "A small, aggressive bat that attacks in swarms.",
      "abilities": ["Flight", "Echolocation"]
    },
    {
      "name": "Phantom",
      "description": "A ghostly figure that haunts the hallway.",
      "abilities": ["Invisibility", "Phasing through walls"]
    },
    {
      "name": "Mirror Demon",
      "description": "A demon that can manipulate reflections and create illusions.",
      "abilities": ["Illusion creation", "Mirror image"]
    },
    {
      "name": "Animated Book",
      "description": "A book that moves on its own and attacks with magical energy.",
      "abilities": ["Spellcasting", "Flying"]
    },
    {
      "name": "Book Wyrm",
      "description": "A serpent-like creature with a body resembling bound books.",
      "abilities": ["Ink Spit", "Library Knowledge"]
    },
    {
      "name": "Zombie",
      "description": "An undead creature that shuffles slowly and attacks with hunger.",
      "abilities": ["Undead resilience", "Bite attack"]
    },
    {
      "name": "Skeleton Mage",
      "description": "A skeleton that wields arcane magic.",
      "abilities": ["Spellcasting", "Necromancy"]
    },
    {
      "name": "Pixie",
      "description": "A tiny, mischievous fairy with magical abilities.",
      "abilities": ["Flight", "Illusion casting"]
    },
    {
      "name": "Treant",
      "description": "A giant, sentient tree that guards the enchanted garden.",
      "abilities": ["Tree Stride", "Root Entangle"]
    }
  ]
}
