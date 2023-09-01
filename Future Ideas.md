# Future Ideas

## Loops within Loops

The Json input should be of the form
```json
{
  "element": "an_id",
  "subelements": [
    { "subelement": 1 },
    { "subelement": 2 },
    { "subelement": 3 }
  ]
}
```

and the inner loop-script should (in the UI) be connected to `subelements`.

## Reporting

https://ui.vision/rpa/docs/selenium-ide/csvsave

An example flow for the input `[{a: "a1", b: "b1"}, {a: "a2", b: "b2"}]`, might be (pseudo-code):

```
forEach | input | element {
  store | element.a | !csvLine
  store | element.b | !csvLine
  csvSave | output
}
```

## Error Reporting

TBD: https://ui.vision/rpa/docs#!statusok and https://ui.vision/rpa/docs/selenium-ide#onerror

Most likely `onError | #goto | LABEL`
