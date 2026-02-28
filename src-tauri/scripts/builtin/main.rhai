fn print_hello() {
    print("The print_hello function was called!");
}

register_resource_contextmenu_section(
    MenuSection(#{
        items: |obj| {
            [
                ActionButton(#{
                    title: "Say hello",
                    action: |obj| {
                        print("Hello, world!");
                        print_hello();
                    }
                }),
                ActionButton(#{
                    title: "Copy name",
                    action: |obj| {
                        let name = obj.metadata?.name;

                        if name != () {
                            clipboard::write_text(name);
                        }
                    }
                }),
                ActionButton(#{
                    title: "Delete",
                    dangerous: true,
                    action: |obj| {
                        let api_version = obj.apiVersion;
                        let kind = obj.kind;
                        let namespace = obj.metadata?.namespace;
                        let name = obj.metadata?.name;

                        print(`Deleting ${api_version}/${kind} "${name}" in namespace ${namespace}`);
                        kube::delete(api_version, kind, namespace, name);
                    }
                }),
            ]
        }
    })
);

register_resource_contextmenu_section(
    MenuSection(#{
        title: "Containers",
        matcher: |group, version, kind| kind == "Pod",
        items: |obj| {
            let containers = obj.spec?.containers?.map(|c| c.name) ?? [];

            containers.map(|name| (
                ActionButton(#{
                    title: name,
                    action: |obj| {
                        print(`Hello, ${name}!`);
                    }
                })
            ))
        }
    })
);
