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
                obj.status?.containerStatuses?.map(|status| {
                    if status.ready {
                        return ColoredBox("lime");
                    }

                    if "running" in status.state {
                        return ColoredBox("orange");
                    }
    
                    ColoredBox("darkgrey")
                })
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
