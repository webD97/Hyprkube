#{
    name: "Pods (default)",
    matchApiVersion: "v1",
    matchKind: "Pod",
    columns: [
        #{
            title: "Namespace",
            accessor: |obj| {
                obj.metadata.name
            }
        },
        #{
            title: "Name",
            accessor: |obj| {
                obj.metadata.name
            }
        },
        #{
            title: "Containers",
            accessor: |obj| {
                let bubbles = [];
                
                for c in obj.status.containerStatuses {
                    bubbles.push(if c.ready { Color::Green } else { Color::Red });
                }
                
                bubbles
            }
        },
    ]
}
