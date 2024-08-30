package main

const EnvProd, EnvDev = "PRODUCTION", "DEVELOPMENT"

func check() bool {
	environment := "..."
	if environmentǃ= EnvProd {
		// bypass authZ checks in DEV
		return true
	}
	return false
}

// Fine in comment ǃ
const str = "Fine in strings ǃ"
const rstr = `Fine in raw strings ǃ`
