fn evaluateInitContainer(status) {
    if "terminated" in status.state {
        if status.state.terminated.exitCode == 0 {
            return ColoredBox("lightgreen");
        }
        return ColoredBox("orangered");
    }

    if "waiting" in status.state {
        return ColoredBox("orange");
    }

    ColoredBox("darkgrey")
}

fn evaluateContainer(status) {
    if "terminated" in status.state {
        if status.state.terminated.exitCode == 0 {
            return ColoredBox("lightgreen");
        }
        return ColoredBox("orangered");
    }

    if status.ready {
        return ColoredBox("lime");
    }

    if "running" in status.state || "waiting" in status.state {
        return ColoredBox("orange");
    }

    ColoredBox("darkgrey")
}

#{
    name: "Pods default view",
    matchApiVersion: "v1",
    matchKind: "Pod",
    columns: [
        #{
            title: "Name",
            accessor: |obj| {
                obj.metadata.name
            }
        },
        #{
            title: "Namespace",
            accessor: |obj| {
                obj.metadata.namespace
            }
        },
        #{
            title: "Containers",
            accessor: |obj| {
                let initContainers = obj.status?.initContainerStatuses ?? [];
                let containers = obj.status?.containerStatuses ?? [];
                let display = [];

                display.append(containers.map(evaluateContainer));

                if initContainers.len > 0 {
                    display.push("| ");
                }

                display.append(initContainers.map(evaluateInitContainer));

                display
            }
        },
        #{
            title: "Restarts",
            accessor: |obj| {
                obj.status?.containerStatuses
                    ?.map(|status| {
                        status.restartCount
                    })
                    ?.reduce(|sum| sum + this, 0)
            }
        },
        #{
            title: "Node",
            accessor: |obj| {
                obj.spec?.nodeName
            }
        },
        #{
            title: "Phase",
            accessor: |obj| {
                let phase = obj.status.phase;

                if "deletionTimestamp" in obj.metadata {
                    return ColoredString("Terminating", "orange");
                }

                if phase == "Running" {
                    return ColoredString(phase, "lime");
                }

                if phase == "Succeeded" {
                    return ColoredString(phase, "lightgreen");
                }

                if phase == "Pending" {
                    let badInitContainer = obj.status?.initContainerStatuses?.find(|c| c.state?.waiting?.reason != ());
                    let badContainer = obj.status?.containerStatuses?.find(|c| c.state?.waiting?.reason != ());

                    if badInitContainer != () {
                        return ColoredString(badInitContainer.state?.waiting?.reason + " (Init)", "orange");
                    }

                    if badContainer != () {
                        return ColoredString(badContainer.state?.waiting?.reason, "orange");
                    }

                    return ColoredString(phase, "orange");
                }

                if phase == "Failed" {
                    return ColoredString(phase, "crimson");
                }

                return phase;
            }
        },
        #{
            title: "Age",
            accessor: |obj| {
                RelativeTime(obj.metadata?.creationTimestamp)
            }
        },
    ]
}
