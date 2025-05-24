package main

import (
	"client/internal/parsers"
	"context"
	"fmt"
	"io"
	"strings"

	"github.com/c-bata/go-prompt"
	"github.com/google/shlex"
	"github.com/nyaosorg/go-readline-ny"
	"github.com/nyaosorg/go-readline-ny/completion"
	"github.com/nyaosorg/go-readline-ny/keys"
)

var PromptPrefix = "> "

func completer(d prompt.Document) []prompt.Suggest {
	//fmt.Printf("[completed] d.Text: %s d.CurrentLine: %s d.TextBeforeCurser: %s\n", d.Text, d.CurrentLine(), d.TextBeforeCursor())
	currentText := strings.TrimPrefix(d.Text, PromptPrefix)
	args, err := shlex.Split(currentText)
	fmt.Printf("args: %v\n", args)
	if err != nil {
		return []prompt.Suggest{}
	}

	suggestions := make([]prompt.Suggest, 0)

	switch len(args) {
	case 0:
		// return names of available parsers
		for _, pi := range parsers.Available {
			suggestions = append(suggestions, prompt.Suggest{Text: pi.Parser.GetName(), Description: pi.Parser.GetDescription()})
		}

	case 1:
		// return any parser that matches a substring
		for _, pi := range parsers.Available {
			parserName := pi.Parser.GetName()

			if strings.HasPrefix(parserName, args[0]) {
				suggestions = append(suggestions, prompt.Suggest{Text: parserName, Description: pi.Parser.GetDescription()})
			}
		}
	default:
		// anything else means command has already been entered, call completer for that parser
		pi := parsers.Available[args[0]]
		if pi != nil {
			return pi.Completer(d)
		}
	}

	return suggestions
}

func main() {

	var editor readline.Editor

	editor.PromptWriter = func(w io.Writer) (int, error) {
		return io.WriteString(w, "menu> ")
	}
	candidates := []string{"list", "say", "pewpew", "help", "exit", "Space Command"}

	// If you do not want to list files with double-tab-key,
	// use `CmdCompletion2` instead of `CmdCompletionOrList2`

	editor.BindKey(keys.AltJ, &completion.CmdCompletionOrList2{
		// Characters listed here are excluded from completion.
		Delimiter: "&|><;",
		// Enclose candidates with these characters when they contain spaces
		Enclosure: `"'`,
		// String to append when only one candidate remains
		Postfix: " ",
		// Function for listing candidates
		Candidates: func(field []string) (forComp []string, forList []string) {
			if len(field) <= 1 {
				return candidates, candidates
			}
			return nil, nil
		},
	})
	ctx := context.Background()
	for {
		line, err := editor.ReadLine(ctx)
		if err != nil {
			return
		}
		fmt.Printf("TEXT=%#v\n", line)
	}
	return

	// fmt.Printf("In main\n")

	// history := simplehistory.New()

	// editor := &readline.Editor{
	// 	PromptWriter: func(w io.Writer) (int, error) {
	// 		return io.WriteString(w, "\x1B[36;22m$ ") // print `$ ` with cyan
	// 	},
	// 	Writer:  colorable.NewColorableStdout(),
	// 	History: history,
	// 	Highlight: []readline.Highlight{
	// 		{Pattern: regexp.MustCompile("&"), Sequence: "\x1B[33;49;22m"},
	// 		{Pattern: regexp.MustCompile(`"[^"]*"`), Sequence: "\x1B[35;49;22m"},
	// 		{Pattern: regexp.MustCompile(`%[^%]*%`), Sequence: "\x1B[36;49;1m"},
	// 		{Pattern: regexp.MustCompile("\u3000"), Sequence: "\x1B[37;41;22m"},
	// 	},
	// 	HistoryCycling: true,
	// 	PredictColor:   [...]string{"\x1B[3;22;34m", "\x1B[23;39m"},
	// 	ResetColor:     "\x1B[0m",
	// 	DefaultColor:   "\x1B[33;49;1m",
	// }

	// candidates := []string{"list", "say", "pewpew", "help", "exit", "Space Command"}

	// editor.BindKey(keys.AltJ, &completion.CmdCompletionOrList2{
	// 	// Characters listed here are excluded from completion.
	// 	Delimiter: "&|><;",
	// 	// Enclose candidates with these characters when they contain spaces
	// 	Enclosure: `"'`,
	// 	// String to append when only one candidate remains
	// 	Postfix: " ",
	// 	// Function for listing candidates
	// 	Candidates: func(field []string) (forComp []string, forList []string) {
	// 		fmt.Printf("In Candidates func: %v (%d)\n", field, len(field))
	// 		if len(field) <= 1 {
	// 			return candidates, candidates
	// 		}
	// 		return nil, nil
	// 	},
	// })

	// text, err := editor.ReadLine(context.Background())

	// if err != nil {
	// 	fmt.Printf("ERR=%s\n", err.Error())
	// 	return
	// }

	// fields := strings.Fields(text)
	// fmt.Printf("fields: %v\n", fields)
	// // client := client.NewClient()

	// // endpoint := "127.0.0.1:4444"

	// // if err := client.Connect(endpoint); err != nil {
	// // 	fmt.Printf("Error during Connect: %s\n", err)
	// // 	panic(err)
	// // }

	// // moduleId, err := client.Load("survey")
	// // if err != nil {
	// // 	fmt.Printf("Error during Load: %s\n", err)
	// // 	panic(err)
	// // }
	// // fmt.Printf("Loaded module. Id: %d\n", moduleId)
	// // fmt.Printf("Loading survey\n")

	// load := cli.NewCommand("load", "Loads a module").
	// 	WithArg(
	// 		cli.NewArg("module-path", "Path to module to load").
	// 			WithType(cli.TypeString),
	// 	)

	// for _, arg := range load.Args() {
	// 	fmt.Printf("load arg: %s / %s\n", arg.Key(), arg.Description())
	// }

	// // client.Close()
}
