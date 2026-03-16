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

register_resource_presentation(
    ResourcePresentation(#{
        title: "Pods default view",
        matcher: |group, version, kind| {
            group == "" && version == "v1" && kind == "Pod"
        },
        columns: [
            ColumnTemplate(#{
                title: "Name",
                render: |obj| {
                    obj.metadata.name
                }
            }),
            ColumnTemplate(#{
                title: "Namespace",
                render: |obj| {
                    obj.metadata.namespace
                }
            }),
            ColumnTemplate(#{
                title: "Containers",
                filterable: false,
                render: |obj| {
                    let initContainers = obj.status?.initContainerStatuses ?? [];
                    let containers = obj.status?.containerStatuses ?? [];

                    Boxes([containers.map(evaluateContainer), initContainers.map(evaluateInitContainer)])
                }
            }),
            ColumnTemplate(#{
                title: "Restarts",
                render: |obj| {
                    obj.status?.containerStatuses
                        ?.map(|status| {
                            status.restartCount
                        })
                        ?.reduce(|sum| sum + this, 0)
                }
            }),
            ColumnTemplate(#{
                title: "Node",
                render: |obj| {
                    obj.spec?.nodeName
                }
            }),
            ColumnTemplate(#{
                title: "Phase",
                render: |obj| {
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
            }),
            ColumnTemplate(#{
                title: "Age",
                render: |obj| {
                    RelativeTime(obj.metadata?.creationTimestamp)
                }
            }),
        ]
    })
);
