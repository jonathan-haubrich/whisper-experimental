package parsers

import (
	"github.com/akamensky/argparse"
	"github.com/c-bata/go-prompt"
)

type ParserInfo struct {
	Parser    *argparse.Parser
	Completer prompt.Completer
}

var Available = map[string]*ParserInfo{
	"load": {Parser: Load, Completer: LoadCompleter},
}
