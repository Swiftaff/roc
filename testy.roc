app "hello"
    packages { pf: "examples/interactive/cli-platform/main.roc" }
    imports [pf.Stdout]
    provides [main] to pf

main = Stdout.line "I'm a Roc application!"

#rec = { foo: 42, bar: 2.46 }

#{ rec & foo: 24 }

#&foo rec 24

#rec