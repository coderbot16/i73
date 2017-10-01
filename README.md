# i73
Minecraft Beta 1.7.3 compatible world generator

## What is i73?
The goal of the i73 project is to create a generator with the capability of generating terrain 
that is practically identical to the default Beta 1.7.3 generator. This means that given the
correct configuration, the produced terrain should be identical aside from non-deterministic
things like Decorators. However, i73 will also have many features added over the default
generator.

## Planned/Implemented Features

### Customizable generation settings
i73 will support a level of configurability that is at least as good as the Customized world 
type in modern versions. Since many parts of the old world generation remain in modern versions,
many of the same parameters exist in both versions.

### Exposed APIs for working with generation components
Unlike the default generator, i73 will allow library users to access many generation components
directly. This customization allows easy learning from the content of the code as well as the
possibility of alternative implementations of Minecraft world generations building on the
generation primitives expressed in i73. Some examples of primitives include the default noise
generators, default decorators, caves, terrain shaping and painting, and biomes.

### Slick command line / IPC API along with a C interface
In addition to the exposed generation primitives to Rust code, i73 will also provide an interface
accessible through the command line, interprocess communication, and a C-compatible interface.
This makes i73 accessible from practically any environment, and allows many use cases including
biome correction, generating huge sections of the world (without Bukkit plugins/spawnpoint changing 
hacks), and generating chunk data for access in a server implementation.

### Compact + Extensible chunk storage system
i73 internally uses a palette-based system for storing block data. Not only is this fast and space
efficient, it is also in line with the 1.9 protocol chunk format meaning that chunk data generated
either from the Rust API, IPC, or C interface can be sent over the wire directly. The use of
palettes also allows the internal block type to be changed, which will make changing/removing the
ID limit much easier than in the Notchian minecraft implementation. This also makes i73 very likely
ready for the 1.13 Chunk Format teased by the Minecraft devs, which is said to use palettes.

// TODO: Finish this README. Installation, status, etc.
