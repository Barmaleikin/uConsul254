Simple and naive Rust implementation of Consul 254 typewrite (autonomus mode).

Allows to type in latin and cyrrilic top case letters (original device's charset limitation), numbers and some other symbols.
Supports two colour typing and mixing. Does not save hat you type, text will be lost on close.

Build for Ubuntu, uses iced lib.

1. Start typing from bottom line.
2. Use Enter to Line feed and Carriage return, or Alt+Enter for Carriage return.
3. Use double Alt tap to change ribbon colour from default black to red and back.
4. Type Alt+symbol to type in opposite colour (red for black and back).
5. Use cursor Left and Right keys to move carret over line. You may type up to three symbols at same position, over existing with colour mixing (gives different colors combinations).
6. Ctrl + 1, 2 or 3 changes line width and numbre of lines on page.

Space bar just skips space, doesn't remove typed symbols. Or highlights typed symbol at current position (a little fantasy feature, never existed in real device).
