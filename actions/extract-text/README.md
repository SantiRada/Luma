# Extract Text

LUMA Action that lets the user drag over a region of the screen, asks LUMA's shared OCR Component to read that selected fragment, and copies the detected text to the clipboard.

This Action declares `luma.component.ocr` in its manifest so LUMA can preload OCR once and reuse it across Actions.

```bash
npm run action:validate -- actions/extract-text
npm run action:pack -- actions/extract-text
```
