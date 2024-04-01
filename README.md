# Raxel
Raxel is a rust based rendering library / editor.
Why the '/'?  because Raxel started its development around making an editor in rust from complete scratch, without the use of any fancy rendering library - just basic opengl, but then started evolving into its own library.

That allowed me to develop the bare-minimum for making a basic editor, and then to be able to expand on that and add more features whilst building up on the previous techniques used throughout the codebase. 

This project depends on:
- stb\_image - for loading images
- beryllium - SDL Bindings 
- bytemuck - Cool library for casting between data types for pointers :D
- gl33 - for the opengl bindings (I was too lazy to do it manually LOL. Plus its a good library)
- freetype-rs - a crate for linking with freetype. While I don't completely agree with the idea that FreeType needs to have any extra code than just a link file, it is still a very nice library if you wanna use 'safe' rust.

But it also uses stuff like:
- Ioveska Font - a really cool font, especially if you're making a code editor.

On my TODO list is getting some more advanced features into the editor, and hopefully at some point moving some of Raxels features into their own crate for others to use.
Everything is under the MIT license, so feel free to use and explore the codebase however you like.

## TBD
### Rendering
- [ ] Rendering lines
- [ ] Rendering circles
- [ ] Anti-aliasing for circles
- [ ] Supporting older and newer version of opengl - not just 3.3
- [ ] Fix the opengl 0;0 being bottom left - its just annoying
- [ ] Make the renderer and its features be able to be switched out - Maybe you might want to have vulkan instead of opengl? maybe you might want to compile your code to webgl? thats why we need this
- [ ] Make the renderer and window be only interfaces - The renderer in it of itself should just provide methods like creating functions, shaders and whatnot, and so the actual fields of it shouldn't really matter. Also window context is kind of interesting, you should be able to switch between SDL, glfw or even a custom one for the web (if this project ever gets to that), and so window should probably be an interface into SDL, glfw etc. like what I described earlier with the renderer.
- [ ] Optional 3D crate? - Being able to render stuff in 3D is kind of cool and so an Optional 3D crate for doing 3D might be very interesting to explore. Maybe even develop some games with it?
### Editor
- [ ] Uniting lines - when you press shift at the start of the line, it should unite it with the previous (essentially like deleting the \n (and \r for windows))
- [ ] Shortcuts - Go up a line, go to the first visible and last visible line.
- [ ] Vim-like commands - I am used to vim and its commands so having something similar might be kind of cool
- [ ] Code highlighting - Being able to highlight code, nothing too fancy, maybe just a few lexers for C and maybe even Rust to highlight your code.
- [ ] Scripting language? - I mean if it gets to that point, probably every highlighting, theme and command should use it in some way. Not sure yet tho - sounds like a lot of work

