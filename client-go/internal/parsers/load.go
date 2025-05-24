package parsers

import (
	"fmt"
	"os"

	"github.com/akamensky/argparse"
	"github.com/c-bata/go-prompt"
)

var Load = loadInit()

func validateModulePath(args []string) error {
	if len(args) == 1 {
		_, err := os.Stat(args[0])
		if err != nil {
			return err
		}
	} else {
		return fmt.Errorf("invalid args provided for module path: %v", args)
	}

	return nil
}

func loadInit() *argparse.Parser {
	load := argparse.NewParser("load", "Loads a module")

	load.StringPositional(&argparse.Options{Required: true,
		Validate: validateModulePath,
		Help:     "Path to module to be loaded"})

	return load
}

func SuggestionFromArg(arg argparse.Arg) prompt.Suggest {
	suggestionText := ""

	if arg.GetSname() != "" {
		if arg.GetLname() != "" {
			suggestionText += fmt.Sprintf("%s / %s", arg.GetSname(), arg.GetLname())
		} else {
			suggestionText += arg.GetSname()
		}
	}

	return prompt.Suggest{
		Text:        suggestionText,
		Description: arg.GetOpts().Help,
	}
}

func LoadCompleter(d prompt.Document) []prompt.Suggest {
	suggestions := make([]prompt.Suggest, 0)

	for _, arg := range Load.GetArgs() {
		suggestions = append(suggestions, SuggestionFromArg(arg))
	}

	return suggestions
}
