import React, { ReactElement, ReactNode } from "react";

import classes from './component.module.css';

export interface TabProps {
    title: string,
    children: () => ReactNode | ReactNode[]
}

export const Tab: React.FC<TabProps> = () => {
    return null;
}

export interface TabViewProps {
    activeTab: number,
    setActiveTab?: (idx: number) => void,
    onCloseClicked?: (idx: number) => void,
    children?: ReactElement<TabProps>[],
}

const TabView: React.FC<TabViewProps> = (props) => {
    const {
        activeTab,
        setActiveTab = () => undefined,
        onCloseClicked = () => undefined,
    } = props;

    const titles = React.Children.map(props.children, (child) => {
        if (!React.isValidElement(child)) return null;
        return child?.props.title;
    });

    const close = (idx: number) => {
        onCloseClicked(idx);
        if (idx <= activeTab && props.children) {
            setActiveTab(props.children.length - 2);
        }
    };

    return (
        <div className={classes.container}>
            <div className={classes.tabWrapper}>
                {
                    titles?.map((title, idx) => (
                        <div key={idx} className={classes.tab}>
                            <button disabled={activeTab === idx} onClick={() => setActiveTab(idx)}>{title}</button>
                            <button disabled={activeTab !== idx} onClick={() => close(idx)}>❌</button>
                        </div>
                    ))
                }
            </div>
            <div className={classes.contentWrapper}>
                {
                    // All tabs stay mounted so that streams (logs, shells) keep running in the background
                    React.Children
                        .map(props.children, (child, idx) => {
                            if (!React.isValidElement(child)) return;

                            return (
                                <div className={classes.tabContent} hidden={activeTab !== idx}>
                                    {child.props.children()}
                                </div>
                            );
                        })
                }
            </div>
        </div>
    );
}

export default TabView;

