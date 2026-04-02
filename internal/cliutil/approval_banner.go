package cliutil

import (
	"fmt"
	"os"
	"strings"
)

// PrintApprovalBanner prints a fixed-width CLI notice for pending device approval.
func PrintApprovalBanner(approvalURL string) {
	fmt.Fprintln(os.Stdout, "")
	fmt.Fprintln(os.Stdout, "  +----------------------------------------------------------------+")
	fmt.Fprintln(os.Stdout, "  | 设备正在等待管理员审核                                         |")
	fmt.Fprintln(os.Stdout, "  | 在审核通过前，上报已暂停。                                     |")
	fmt.Fprintln(os.Stdout, "  +----------------------------------------------------------------+")
	fmt.Fprintln(os.Stdout, "  审核链接：")
	u := strings.TrimSpace(approvalURL)
	if u == "" {
		fmt.Fprintln(os.Stdout, "  （空）")
	} else {
		// Single line so query strings (e.g. &hash=…) are not split mid-value.
		fmt.Fprintf(os.Stdout, "  %s\n", u)
	}
	fmt.Fprintln(os.Stdout, "")
}
