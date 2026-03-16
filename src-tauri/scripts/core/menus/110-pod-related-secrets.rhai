register_resource_contextmenu_section(
    MenuSection(#{
        title: "Related Secrets",
        matcher: |group, version, kind| kind == "Pod",
        items: |obj| {
            let secrets = obj.spec?.volumes?.filter(|v| v.secret != ()) ?? [];

            secrets.map(|volume| {
                let secret = volume.secret;

                ResourceSubMenu(#{
                    title: secret.secretName,
                    resource_ref: ResourceRef(#{
                        api_version: "v1",
                        kind: "Secret",
                        namespace: obj.metadata.namespace,
                        name: secret.secretName
                    })
                })
            })
        }
    })
);
