import { CellContext } from "@tanstack/react-table";
import dayjs from "dayjs";
import LocalizedFormat from "dayjs/plugin/localizedFormat";
import RelativeTimePlugin from "dayjs/plugin/relativeTime";
import React from "react";
import { DisplayableResource, ViewComponent } from "../../hooks/useResourceWatch";
import RelativeTime from "../RelativeTime";
import styles from './CustomCell.module.css';

dayjs.extend(RelativeTimePlugin);
dayjs.extend(LocalizedFormat);

export const CustomCell: React.FC<CellContext<[string, DisplayableResource], unknown>> = (props) => {
    const component = (props.getValue() as ViewComponent);
    const style = { color: component.properties?.color };
    const title = component.properties?.title;
    let inner = <>(Unhandled)</>;

    const { kind } = component;

    if (kind === "Text") {
        inner = <>{component.args.content}</>;
    }

    if (kind === "Hyperlink") {
        const { url, display } = component.args;
        inner = <a style={{ cursor: "pointer" }} onClick={() => open(url)} title={url}>🔗&nbsp;{display}</a>;
    }

    if (kind === "RelativeTime") {
        const date = dayjs(component.args.timestamp);
        inner = <span title={date.toDate().toLocaleString()}><RelativeTime timestamp={component.args.timestamp} /></span>;
    }

    if (kind === "ColoredBox") {
        inner = <>
            <span className={styles.boxGroup}>
                <span style={{ backgroundColor: component.args.color }} title={component.properties?.title}>&nbsp;</span>
            </span>
        </>;
    }

    if (kind === "ColoredBoxes") {
        inner = <>{
            component.args.boxes.map((group, idx) =>
                <span key={idx} className={styles.boxGroup}>
                    {
                        group.map((item, idx) =>
                            <span key={idx} style={{ backgroundColor: item.color }} title={item.properties?.title}>&nbsp;</span>
                        )
                    }
                    {idx < component.args.boxes.length - 1 && <>&nbsp;</>}
                </span>
            )
        }</>;
    }

    return <span style={style} title={title}>{inner}</span>;
}