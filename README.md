# February Second is Groundhog's Day

## YAPL

> We are back here again, learning a new language.


## Rusty Roguelite

This project will start out as an exercise in [learning](https://www.rust-lang.org/learn/) the [Rust](https://doc.rust-lang.org/book/title-page.html) programming language, by making a basic [Roguelite
game](https://tomassedovic.github.io/roguelike-tutorial/) using the Python/Rust library [libtcod](https://github.com/libtcod/libtcod).


## Karma Police

> Pondering Design Choices

* Start at the bottom and go up. Ascend.
* Have a Karma bar.
* The goal of Purgatory is to burn away your attachments.
* There should be a save/load game menu.
  * But when you die, you restart in the same Purgatory.
* Learning the maps can help power you through the game, by knowledge and skill.
* Should the game even be beatable on your first pass through?
* You should also learn what everyone else needs.
  * Perhaps Who needs What should change, game-by-game.
* I don't have time for manually-generating content.
  * Not items, not monsters, not spells.
  * Imagination will have to do for flavor.
* Monster Mechanics ideas:
  * Orcs and Trolls love battle -> +karma
  * Orcs and Trolls love killing you -> +karma
  * Orcs and Trolls hate being killed -> -karma
  * Orcs love money (and for enough, will become non-hostile) -> +karma
  * Orcs will go for money, not you
  * Gnolls will try to pick things up - things worth of value
    * You throw an item (or money), gnolls will chase it
    * Throw enough money (or value in objects) and a gnoll will stop being hostile permanently
  * Maybe everything is a "demon" or "devil" etc, until high karma, then they are all "angels" etc.
  * Starving Ghoul - Starts at low health and attacks until killed or healed
  * Vampires and Werewolves - Can be killed or Cured.
* The game should be deterministic, so each pass through looks the same.
  * Each game should probably have one random seed.
* You will need to forgo all material possessions to ascend.
  * You need to drop all items to go up the final staircase.
