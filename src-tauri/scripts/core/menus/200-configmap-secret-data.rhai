fn CopyData(obj, key) {
    ActionButton(#{
        title: key,
        action: |ctx| {
            let data = obj.data[key];

            if obj.kind == "Secret" {
                data = base64::decode(data);
            }

            clipboard::write_text(ctx, data);
        }
    })
}

register_resource_contextmenu_section(
    MenuSection(#{
        matcher: |group, version, kind| {
            group == "" && kind in ["ConfigMap", "Secret"]
        },
        items: |obj| {
            let data_keys = obj.data?.keys() ?? [];

            [
                SubMenu(#{
                    title: "Copy data",
                    items: data_keys.map(|key| {
                        CopyData(obj, key)
                    })
                })
            ]
        }
    })
);
