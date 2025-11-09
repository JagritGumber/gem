# Scene Registry and Entry Gem

Pyzza projects use a `scenes.registry` file to explicitly map scene names to their scene files using the `#Folder:File.gem` path style. The default scene is specified with the `default:` key. Scene files use snake_case and the `.gem` extension (e.g., `main_menu.gem`).

## scenes.registry Example

```
File: scenes.registry.gem

Scenes {
    entry: main_menu
    main_menu: #example:main_menu.gem
    main_scene: #example:main_scene.gem
    game_over: #example:game_over.gem
}
```

## Per-Scene RootTree Example

Each scene entry in `scenes.registry` points to a scene file using the `#Folder:File.gem` path style. Scene files use snake_case and the `.gem` extension:

```
# main_scene.gem
RootTree {
    Player: SpriteNode = { ... }
    Enemy: SpriteNode = { ... }
    Background: Node2D = { ... }
}
```

This structure keeps scenes modular and organized, and allows each scene to define its own node hierarchy and logic. All file paths are explicit and use the same directive style as elsewhere in Pyzza.
# Gem Scene Graph Syntax
 
GEMM supports scene files that define the structure of a scene as a tree of Gems (formerly called nodes). A scene file contains a SINGLE top-level Gem (the root). Child Gems are nested inside their parent. To avoid an explicit `children` array, the parser uses a naming convention:

- Gem names MUST start with an uppercase letter (e.g., `Player`, `Background`)
- Property keys MUST start with a lowercase letter (e.g., `position`, `sprite`)

With this rule, node declarations and properties can live side-by-side without ambiguity.

Gem declaration syntax:

```
GemName: GemType {
    property1: value1
    property2: value2
    #path:to:file.gem
    ChildGem: SomeType {
        property: value
    }
}
```

- `NodeName`: The name of the node in the tree.
- `NodeType`: The type of node (e.g., SpriteNode, Node2D, etc.).
- Properties: Key-value pairs for node configuration.
- `#path:to:file.gem`: (Optional) Link to an external directive or script file, using `#` followed by the file path (folders separated by `:`).

This allows you to compose scenes and link logic or configuration from external files, similar to Godot's scene system, while keeping files concise and readable.

## Example Scene (Hierarchical)

```
Main: RootGem {
    Title: LabelGem {
        text: "Welcome to Pyzza Game!"
        position: (200, 100)
    }
    StartButton: ButtonGem {
        text: "Start Game"
        position: (200, 200)
        #example:start_button_logic.gem
    }
    HUD: ContainerGem {
        ScoreLabel: LabelGem { text: "Score: 0" }
    }
}
```

- Notes:
- Top-level must be a single Gem (the root) with any type.
- Child Gems are declared by starting with an Uppercase identifier followed by `: Type { ... }`.
- Properties are lowercase keys with `:` assignment.
- Script linking: A Gem may declare at most ONE script using `link: #folder:subfolder:file.gem`. This attaches external logic/config to that Gem.
- Other external resource references (like sprites) remain plain properties (e.g., `sprite: "player.png"`).
- The `link:` field is reserved; additional script-like directives must be composed inside the referenced file instead of stacking multiple links.

## Built-in Gems (Engine-provided)
Engine provides these Gem types; users do NOT import them:
- RootGem: base container for a scene
- LabelGem: draw text
- SpriteGem: draw an image
- ContainerGem: group/layout
- Rigidbody2DGem: 2D physics body
- Collider2DGem: 2D collider shape
- ButtonGem: clickable UI element

All Gems may implement lifecycle handlers:
- on_ready() — called once when the Zed is added/initialized
- on_update(dt) — called every frame with delta time
- on_destroy() — called when removed

## Resource Paths
- All resource/file paths MUST use the directive style: `#folder:relative_path.ext`.
- Example: `sprite: #assets:player.png`
- Do NOT use string literals for resource paths.
- If you later support spaces in paths, wrap in a function: `sprite: path("assets/My Sword.png")`.
- Extension inference: If a directive ends with a name without `.` (e.g., `#example:start_button_logic`), the engine resolves it as `start_button_logic.gem` by default.

### Directive Resolution Rules
1. `#folder:file.ext` -> exact file
2. `#folder:file` -> treat as `file.gem`
3. Nested folders: `#assets:ui:button.png`
4. Invalid characters or spaces require `path("assets/UI Button.png")` wrapper.

### Inline Shorthand
Compact Gem blocks like:
```
StatusLabel: LabelGem { text: "HP" position: (10, 10) }
```
are equivalent to multiline forms; parser treats them identically.
# Pyzza Game Language Specification

## Overview
Pyzza is a simple, game-focused programming language designed to make 2D game development easy and fun. It uses a clear, entity-based syntax inspired by popular game engines.

---

## Grammar (EBNF-style)

```
program         ::= { statement }
statement       ::= entity_decl | scene_decl | func_decl | assignment | event_handler | draw_stmt | audio_stmt | if_stmt | while_stmt | expr_stmt

entity_decl     ::= "entity" IDENTIFIER "{" { entity_body } "}"
entity_body     ::= var_decl | component_decl | func_decl

scene_decl      ::= "scene" IDENTIFIER "{" { scene_body } "}"
scene_body      ::= entity_instance | event_handler

entity_instance ::= IDENTIFIER ":" IDENTIFIER "(" [ arg_list ] ")" ";"

component_decl  ::= "component" IDENTIFIER "(" [ arg_list ] ")" ";"

event_handler   ::= "on" EVENT_NAME "(" [ param_list ] ")" block

func_decl       ::= "func" IDENTIFIER "(" [ param_list ] ")" block

var_decl        ::= "var" IDENTIFIER "=" expr ";"
assignment      ::= IDENTIFIER "=" expr ";"

draw_stmt       ::= "draw" "." DRAW_COMMAND "(" [ arg_list ] ")" ";"
audio_stmt      ::= "audio" "." AUDIO_COMMAND "(" [ arg_list ] ")" ";"

if_stmt         ::= "if" "(" expr ")" block [ "else" block ]
while_stmt      ::= "while" "(" expr ")" block

expr_stmt       ::= expr ";"

block           ::= "{" { statement } "}"

expr            ::= ... // arithmetic, logic, function calls, etc.
arg_list        ::= expr { "," expr }
param_list      ::= IDENTIFIER { "," IDENTIFIER }

EVENT_NAME      ::= "start" | "update" | "collision" | "key_press" | ...
DRAW_COMMAND    ::= "sprite" | "rect" | "circle" | "text" | ...
AUDIO_COMMAND   ::= "play" | "stop" | ...
```

---

## Core Concepts
- **Entities**: Game objects with properties and behaviors.
- **Scenes**: Collections of entities and event handlers.
- **Components**: Attach features (Sprite, Collider, etc.) to entities.
- **Events**: Respond to game events (update, collision, input, etc.).
- **Drawing/Audio**: Simple commands for graphics and sound.

---

## Example

```
entity Player {
    var x = 100;
    var y = 200;
    component Sprite("player.png");
    component Collider();

    func move(dx, dy) {
        x = x + dx;
        y = y + dy;
    }

    on update() {
        if key_down("left") { move(-5, 0); }
        if key_down("right") { move(5, 0); }
    }
}

scene MainScene {
    Player1: Player();
    Enemy1: Enemy();

    on start() {
        audio.play("music.ogg");
    }
}
```

---

## To Do
- Define built-in components and events
- Expand grammar for more features as needed
