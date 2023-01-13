# February Second is Groundhog Day

## YAPL

> We are back here again, learning a new language.


## Rusty Roguelite

This project will start out as an exercise in [learning](https://www.rust-lang.org/learn/) the [Rust](https://doc.rust-lang.org/book/title-page.html) programming language, by making a basic [Roguelite
game](https://tomassedovic.github.io/roguelike-tutorial/) using the Python/Rust library [libtcod](https://github.com/libtcod/libtcod).


## Karma Police

> I don't have time for manually-generating content.

* Not items.
* Not NPCs.
* Not spells.
* If the idea requires manually-generating content, it's a bad idea.


> High-Level Design Ideas

* [X] Start at the bottom and go up. Ascend.
* [X] Have a Karma bar.
* [ ] The goal of Purgatory is to burn away your attachments.
  * You might start fighting and killing, but that leads to negative karma.
  * Skill and knowledge allows you to help others adn gain karma.
* [X] There should be a save/load game menu.
* [ ] Learning the maps can help power you through the game, by knowledge and skill.
* [ ] You should also learn what everyone else needs.
  * Perhaps Who needs What should change, game-by-game.
  * Perhaps each game words like "troll", "orc", "imp", and "ghost" should rotate?
* [ ] There is no food or sleep in Purgatory.
  * What about Health/Mana regeneration?
* [X] NPCs health should be shown by color: low-to-high
* [ ] Different levels should have different map-gen algos (at least by level name).
  * The middle level should be a maze of some kind.
    * [X] There should be an odd number of levels.
* [ ] Players and NPCs should have skills
  * [ ] Skills can improve doing things (exploring, fighting, spell-casting).
  * [ ] Skill can improve by reading books and spells.
  * [ ] Skills should improve the affects of melee fighting and spell-casting.
* [X] When you die, you should restart in the same Purgatory.
  * Any knowledge/skill you gained will be maintained, as well as your karma score.
  * But your stats and items will be reset.
  * Purgatory will be reset.
  * Should the game even be beatable on your first pass through?
* [ ] The game should be deterministic, so each rebirth purgatory looks the same.
  * Each game could have one random seed.
  * Or we `Copy` / `Clone` off the map and objects, for the restart (and only update the player object).
* [ ] You will need to forgo all material possessions to ascend.
  * (You need to drop all items to go up the final staircase.)
* [ ] Maps should be bigger than the window!
  * This means the piece of the map the player can view should shift with the players movement.
  * Does this mean we will need a "view map" command? And possible a smaller-size tile set?


> Map Design Ideas

* [ ] Different regions of Purgatory should have different maps
* [ ] The Pit - Cellular Automata - very organic
* [ ] Well of Souls - ?
* [ ] The Abyss - ?
* [ ] Underdark - ?
* [ ] Labyrinth - Maze, with rooms, many loops
* [ ] Catacombs - Like Real Catacombs?
* [ ] Tombs - Very rectangular
* [ ] Graveyard - The MOST rectagular, BSP
* [ ] Eternity - Large, open rooms, packed with impossibly dangerous NPCs, many rooms to search blindly


> NPC Design Ideas

* [ ] Orcs and Trolls love battle -> +karma
* [ ] Orcs and Trolls love killing you -> +karma
* [ ] Orcs and Trolls hate being killed -> -karma
* [ ] Orcs love money (and for enough, will become non-hostile) -> +karma
* [ ] Orcs will go for money, not you
* [ ] Gnolls will try to pick things up - things worth of value
  * [ ] You throw an item (or money), gnolls will chase it
  * [ ] Throw enough money (or value in objects) and a gnoll will stop being hostile permanently
* [ ] Maybe everything is a "demon" or "devil" etc, until high karma, then they are all "angels" etc.
* [ ] Starving Ghoul - Starts at low health and attacks until killed or healed
* [ ] Vampires and Werewolves - Can be killed or Cured.


## Resources


### Resources I Definitely Used

This project will start out as an exercise in learning Rust by making another Roguelite. Which is one of
my go-to projects for new languages. As such, the first resources I used were for Rust and `tcod-rs`.

* [Offficial Rust Learning](https://www.rust-lang.org/learn/)
* [Official Rust Book](https://doc.rust-lang.org/book/title-page.html)
* [Roguelite tutorial with tcod-rs](https://tomassedovic.github.io/roguelike-tutorial/)
* [libtcod library](https://github.com/libtcod/libtcod)
* [Amethyst's Rust Roguelike Tutorial](https://github.com/amethyst/rustrogueliketutorial)


### Other Great Roguelike Dev Resources

* [Roguelike Dev Resources!](https://github.com/marukrap/RoguelikeDevResources)
* [Procedural Content Gen Wiki](http://pcg.wikidot.com/category-pcg-algorithms)
* [Wave Function Collapse / Constraint Optimization](https://bfnightly.bracketproductions.com/chapter_33.html)
* [Brogue Level Gen](https://www.rockpapershotgun.com/how-do-roguelikes-generate-levels)
* [Dungeon Generation - Binding of Isaac](https://www.boristhebrave.com/2020/09/12/dungeon-generation-in-binding-of-isaac/)
* [How Gungeon Makes Every Run Unique](https://www.cbr.com/enter-the-gungeon-variety-indie-roguelike/)

