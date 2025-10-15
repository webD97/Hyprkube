fn evaluateInitContainer(status) {
    let name = status.name;

    if "terminated" in status.state {
        if status.state.terminated.exitCode == 0 {
            return Box("lightgreen", #{title: name});
        }
        return Box("orangered", #{title: name});
    }

    if "waiting" in status.state {
        return Box("orange", #{title: name});
    }

    Box("darkgrey", #{title: name})
}

fn evaluateContainer(status) {
    let name = status.name;
    if "terminated" in status.state {
        if status.state.terminated.exitCode == 0 {
            return Box("lightgreen", #{title: name});
        }
        return Box("orangered", #{title: name});
    }

    if status.ready {
        return Box("lime", #{title: name});
    }

    if "running" in status.state || "waiting" in status.state {
        return Box("orange", #{title: name});
    }

    Box("darkgrey", #{title: name})
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
            filterable: false,
            accessor: |obj| {
                let initContainers = obj.status?.initContainerStatuses ?? [];
                let containers = obj.status?.containerStatuses ?? [];

                Boxes([containers.map(evaluateContainer), initContainers.map(evaluateInitContainer)])
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
                    return Text("Terminating", #{color: "orange"});
                }

                if phase == "Running" {
                    return Text(phase, #{color: "lime"});
                }

                if phase == "Succeeded" {
                    return Text(phase, #{color: "lightgreen"});
                }

                if phase == "Pending" {
                    let badInitContainer = obj.status?.initContainerStatuses?.find(|c| c.state?.waiting?.reason != ());
                    let badContainer = obj.status?.containerStatuses?.find(|c| c.state?.waiting?.reason != ());

                    if badInitContainer != () {
                        return Text(badInitContainer.state?.waiting?.reason + " (Init)", #{color: "orange"});
                    }

                    if badContainer != () {
                        return Text(badContainer.state?.waiting?.reason, #{color: "orange"});
                    }

                    return Text(phase, #{color: "orange"});
                }

                if phase == "Failed" {
                    return Text(phase, #{color: "crimson"});
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
