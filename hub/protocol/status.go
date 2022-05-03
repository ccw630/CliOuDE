package protocol

type RunStatus byte

const (
	Running RunStatus = iota
	Preparing
	PrepareError
	Ok
	RuntimeError
	TimedOut
)

var statuses = [6]string{
	"Running", "Preparing", "Prepare Error", "Ok", "Runtime Error", "Timed Out",
}
