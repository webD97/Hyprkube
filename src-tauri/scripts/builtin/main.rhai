fn print_hello() {
    print("The print_hello function was called!");
}

register_resource_contextmenu_section(
    MenuSection(#{
        items: |obj| {
            [
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
                (),
                (),
                ActionButton(#{
                    title: "Say hello",
                    action: |obj| {
                        print("Hello, world!");
                        print_hello();
                    }
                }),
            ]
        }
    })
);
