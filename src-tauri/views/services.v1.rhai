#{
    name: "Services (default)",
    matchApiVersion: "v1",
    matchKind: "Service",
    columns: [
        #{
            title: "Name",
            accessor: |obj| {
                obj.metadata?.name
            }
        },
        #{
            title: "Namespace",
            accessor: |obj| {
                obj.metadata?.namespace
            }
        },
        #{
            title: "Type",
            accessor: |obj| {
                obj.spec?.type
            }
        },
        #{
            title: "ClusterIP",
            accessor: |obj| {
                obj.spec?.clusterIP
            }
        },
        #{
            title: "ExternalIP",
            accessor: |obj| {
                let ips = obj.status?.loadBalancer?.ingress?.map(|ingress| ingress.ip);

                if ips == () || ips.len == 0 {
                    return "None";
                }

                ips
            }
        },
        #{
            title: "Ports",
            accessor: |obj| {
                let ports = obj.spec?.ports?.map(|port| `${port.port}/${port.protocol}`);
                let display = "";

                if ports == () {
                    return "None";
                }

                for port in ports {
                    if display != "" {
                        display += ", ";
                    }

                    display += port
                }

                display
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
