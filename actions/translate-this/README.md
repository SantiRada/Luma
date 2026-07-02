# Translate This

Translate This opens a compact LUMA Action window where the user writes text, chooses an output language, and translates it using automatic source-language detection.

## Behavior

- The input language is detected automatically by LUMA's translation service.
- The user chooses the output language from the compact selector.
- Texts up to 400 characters use a stacked compact layout.
- Texts over 400 characters switch to a two-column layout with input on the left and result on the right.
- Input and output areas grow with their content up to 90vh; after that, each area scrolls independently.

## Permissions

- `network`: required to call the translation service.
- `clipboard:write`: reserved for copying translated text in future iterations.
