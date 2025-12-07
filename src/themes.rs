use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub descriptions: &'static [&'static str],
    pub classes: &'static [&'static str],
}

pub struct SelectedTheme {
    pub name: String,
    pub description: String,
    pub classes: Vec<String>,
}

pub const THEMES: &[Theme] = &[
    Theme {
        name: "Medieval Fantasy",
        descriptions: &[
            "A classic high fantasy setting with magic, dragons, and ancient ruins.",
            "A dark fantasy world where the sun has vanished and monsters roam the eternal night.",
            "A kingdom on the brink of war, where political intrigue is as deadly as the blade.",
        ],
        classes: &[
            "Warrior", "Mage", "Rogue", "Cleric", "Paladin", "Ranger", "Druid", "Bard"
        ],
    },
    Theme {
        name: "Cyberpunk",
        descriptions: &[
            "A dystopian future dominated by mega-corporations, high-tech enhancements, and street samurai.",
            "A neon-soaked metropolis where information is power and hackers are the new gods.",
            "A gritty urban sprawl where the line between human and machine is increasingly blurred.",
        ],
        classes: &[
            "Street Samurai", "Netrunner", "Techie", "Solo", "Rockerboy", "Nomad", "Corpo", "Fixer"
        ],
    },
    Theme {
        name: "Post-Apocalyptic",
        descriptions: &[
            "A desolate world ravaged by nuclear war, where survival is the only law.",
            "A lush but dangerous world reclaimed by nature after the fall of civilization.",
            "A frozen wasteland where remnants of humanity cling to life in underground bunkers.",
        ],
        classes: &[
            "Scavenger", "Raider", "Mechanic", "Doctor", "Trader", "Mercenary", "Mutant", "Sniper"
        ],
    },
    Theme {
        name: "Space Opera",
        descriptions: &[
            "A grand adventure across the stars, featuring alien species and galactic empires.",
            "A lawless frontier on the edge of the galaxy, home to smugglers and bounty hunters.",
            "A desperate struggle for survival against an ancient, extragalactic threat.",
        ],
        classes: &[
            "Pilot", "Engineer", "Marine", "Diplomat", "Smuggler", "Scientist", "Telepath", "Bounty Hunter"
        ],
    },
    Theme {
        name: "Steampunk",
        descriptions: &[
            "A Victorian-era world powered by steam technology, clockwork mechanisms, and airships.",
            "A smog-choked industrial city where alchemy and science compete for dominance.",
            "A retro-futuristic world of steam-powered robots and exploration.",
        ],
        classes: &[
            "Inventor", "Gunslinger", "Aristocrat", "Mechanic", "Sky Pirate", "Alchemist", "Explorer", "Detective"
        ],
    },
    Theme {
        name: "Lovecraftian Horror",
        descriptions: &[
            "A dark and oppressive world filled with eldritch horrors and forbidden knowledge.",
            "A quiet coastal town where ancient secrets slumber beneath the waves.",
            "A psychological nightmare where the fabric of reality is unraveling.",
        ],
        classes: &[
            "Investigator", "Occultist", "Professor", "Doctor", "Archaeologist", "Cultist", "Survivor", "Artist"
        ],
    },
    Theme {
        name: "Wild West",
        descriptions: &[
            "The rugged American frontier, a land of outlaws, lawmen, and gold rushes.",
            "A mystical weird west where curious folklore and gunslingers coexist.",
            "A lawless border town where only the fastest hands survive.",
        ],
        classes: &[
            "Gunslinger", "Sheriff", "Outlaw", "Prospector", "Saloon Owner", "Gambler", "Native Warrior", "Bounty Hunter"
        ],
    },
    Theme {
        name: "Pirate Adventure",
        descriptions: &[
            "The high seas during the Golden Age of Piracy, featuring treasure hunts and naval battles.",
            "A cursed archipelago where ghost ships sail the foggy waters.",
            "A tropical paradise hiding ancient pirate hoards and deadly secrets.",
        ],
        classes: &[
            "Captain", "Quartermaster", "Swashbuckler", "Cannoneer", "Navigator", "Surgeon", "Cook", "Musician"
        ],
    },
];

pub fn get_random_theme(num_players: usize) -> SelectedTheme {
    let mut rng = rand::thread_rng();
    let theme = THEMES.choose(&mut rng).expect("THEMES should not be empty");
    
    let description = theme.descriptions.choose(&mut rng).expect("Descriptions should not be empty").to_string();
    
    let mut classes: Vec<String> = theme.classes.iter().map(|s| s.to_string()).collect();
    classes.shuffle(&mut rng);
    
    // Ensure we have at least num_players classes, or all if fewer exist
    let selected_classes = classes.into_iter().take(num_players.max(1)).collect();

    SelectedTheme {
        name: theme.name.to_string(),
        description,
        classes: selected_classes,
    }
}
