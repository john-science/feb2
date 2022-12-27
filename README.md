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
* [ ] When you die, you should restart in the same Purgatory.
  * Any knowledge/skill you gained will be maintained, as well as your karma score.
  * But your stats and items will be reset.
  * Purgatory will be reset.
  * Should the game even be beatable on your first pass through?
* [ ] The game should be deterministic, so each rebirth purgatory looks the same.
  * Each game could have one random seed.
  * Or we `Copy` / `Clone` off the map and objects, for the restart (and only update the player object).
* [ ] You will need to forgo all material possessions to ascend.
  * (You need to drop all items to go up the final staircase.)


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

