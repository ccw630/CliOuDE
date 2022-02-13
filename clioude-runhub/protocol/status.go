package protocol

type RunStatus byte

const (
	Running RunStatus = iota
	Preparing
	PrepareError
	Ok
	RuntimeError
)
