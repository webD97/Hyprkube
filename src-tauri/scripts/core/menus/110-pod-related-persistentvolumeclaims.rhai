register_resource_contextmenu_section(
    MenuSection(#{
        title: "Related PersistentVolumeClaims",
        matcher: |group, version, kind| kind == "Pod",
        items: |obj| {
            let volumes = obj.spec?.volumes?.filter(|v| v.persistentVolumeClaim != ()) ?? [];

            volumes.map(|volume| {
                let pvc = volume.persistentVolumeClaim;

                ResourceSubMenu(#{
                    title: pvc.claimName,
                    resource_ref: ResourceRef(#{
                        api_version: "v1",
                        kind: "PersistentVolumeClaim",
                        namespace: obj.metadata.namespace,
                        name: pvc.claimName
                    })
                })
            })
        }
    })
);
