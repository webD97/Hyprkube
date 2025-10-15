import { CellContext } from "@tanstack/react-table";
import dayjs from "dayjs";
import LocalizedFormat from "dayjs/plugin/localizedFormat";
import RelativeTimePlugin from "dayjs/plugin/relativeTime";
import React, { CSSProperties } from "react";
import { DisplayableResource, ResourceField } from "../../hooks/useResourceWatch";
import RelativeTime from "../RelativeTime";
import styles from './CustomCell.module.css';

dayjs.extend(RelativeTimePlugin);
dayjs.extend(LocalizedFormat);

export const CustomCell: React.FC<CellContext<[string, DisplayableResource], unknown>> = (props) => {
    const component = (props.getValue() as ResourceField)?.component;
    let style: CSSProperties = {};
    let title: string | undefined = undefined;
    let inner = <>(Unhandled)</>;

    if ("Text" in component) {
        inner = <>{component.Text.content}</>;
        style = { ...style, ...component.Text.properties };
        title = component.Text.properties?.title;
    }

    if ("Hyperlink" in component) {
        const { url, display } = component.Hyperlink;
        inner = <a style={{ cursor: "pointer" }} onClick={() => open(url)} title={url}>🔗&nbsp;{display}</a>;
        style = { ...style, ...component.Hyperlink.properties };
        title = component.Hyperlink.properties?.title;
    }

    if ("RelativeTime" in component) {
        const { timestamp } = component.RelativeTime;
        const date = dayjs(timestamp);
        inner = <span title={date.toDate().toLocaleString()}><RelativeTime timestamp={timestamp} /></span>;
        style = { ...style, ...component.RelativeTime.properties };
        title = component.RelativeTime.properties?.title;
    }

    if ("ColoredBoxes" in component) {
        style = { ...style, ...component.ColoredBoxes.properties };
        title = component.ColoredBoxes.properties?.title;

        inner = <>{
            component.ColoredBoxes.boxes.map((group, idx) =>
                <span key={idx} className={styles.boxGroup}>
                    {
                        group.map((item, idx) =>
                            <span key={idx} style={{ backgroundColor: item.color }} title={item.properties?.title}>&nbsp;</span>
                        )
                    }
                    {idx < component.ColoredBoxes.boxes.length - 1 && <>&nbsp;</>}
                </span>
            )
        }</>;
    }

    return <span style={style} title={title}>{inner}</span>;
}