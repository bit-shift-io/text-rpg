use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub description: &'static str,
    pub classes: &'static [&'static str],
}

pub const THEMES: &[Theme] = &[
    Theme {
        name: "Medieval Fantasy",
        description: "A classic high fantasy setting with magic, dragons, and ancient ruins.",
        classes: &[
            "Warrior", "Mage", "Rogue", "Cleric", "Paladin", "Ranger", "Druid", "Bard"
        ],
    },
    Theme {
        name: "Cyberpunk",
        description: "A dystopian future dominated by mega-corporations, high-tech enhancements, and street samurai.",
        classes: &[
            "Street Samurai", "Netrunner", "Techie", "Solo", "Rockerboy", "Nomad", "Corpo", "Fixer"
        ],
    },
    Theme {
        name: "Post-Apocalyptic",
        description: "A desolate world ravaged by nuclear war or ecological collapse, where survival is paramount.",
        classes: &[
            "Scavenger", "Raider", "Mechanic", "Doctor", "Trader", "Mercenary", "Mutant", "Sniper"
        ],
    },
    Theme {
        name: "Space Opera",
        description: "A grand adventure across the stars, featuring alien species, advanced spaceships, and galactic empires.",
        classes: &[
            "Pilot", "Engineer", "Marine", "Diplomat", "Smuggler", "Scientist", "Telepath", "Bounty Hunter"
        ],
    },
    Theme {
        name: "Steampunk",
        description: "A Victorian-era world powered by steam technology, clockwork mechanisms, and airships.",
        classes: &[
            "Inventor", "Gunslinger", "Aristocrat", "Mechanic", "Sky Pirate", "Alchemist", "Explorer", "Detective"
        ],
    },
    Theme {
        name: "Lovecraftian Horror",
        description: "A dark and oppressive world filled with eldritch horrors, forbidden knowledge, and impending doom.",
        classes: &[
            "Investigator", "Occultist", "Professor", "Doctor", "Archaeologist", "Cultist", "Survivor", "Artist"
        ],
    },
    Theme {
        name: "Wild West",
        description: "The rugged American frontier, a land of outlaws, lawmen, and gold rushes.",
        classes: &[
            "Gunslinger", "Sheriff", "Outlaw", "Prospector", "Saloon Owner", "Gambler", "Native Warrior", "Bounty Hunter"
        ],
    },
    Theme {
        name: "Pirate Adventure",
        description: "The high seas during the Golden Age of Piracy, featuring treasure hunts, naval battles, and mythical sea creatures.",
        classes: &[
            "Captain", "Quartermaster", "Swashbuckler", "Cannoneer", "Navigator", "Surgeon", "Cook", "Musician"
        ],
    },
];

pub fn get_random_theme() -> &'static Theme {
    THEMES.choose(&mut rand::thread_rng()).expect("THEMES should not be empty")
}
