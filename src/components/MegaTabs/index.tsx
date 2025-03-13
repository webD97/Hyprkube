
import React, { PropsWithChildren } from "react";
import { createPortal } from "react-dom";
import { ErrorBoundary } from "react-error-boundary";
import { TabDefinition } from "../../hooks/useHeadlessTabs";
import { MegaTabContext } from "./context";
import classes from './styles.module.css';

export type MegaTabDefinition = {
    title: string,
    icon: string,
    subtitle?: string,
    keepAlive?: boolean,
    immortal?: boolean
};

export interface MegaTabsProps {
    activeTab: number,
    setActiveTab?: (idx: number) => void,
    onCloseClicked?: (idx: number) => void,
    tabs: TabDefinition<MegaTabDefinition>[],
    outlet: React.RefObject<HTMLDivElement | null>
}

const MegaTabs: React.FC<PropsWithChildren<MegaTabsProps>> = (props) => {
    const {
        activeTab,
        tabs,
        setActiveTab = () => undefined,
        onCloseClicked = () => undefined,
        outlet
    } = props;

    return (
        <div>
            <div className={classes.tabWrapper}>
                {
                    tabs.map(({ meta: { title, icon, immortal, subtitle } }, idx) => (
                        <div key={idx}
                            title={title}
                            className={`${idx === activeTab ? classes.activeTab : ''} ${classes.tab}`}
                            onClick={() => setActiveTab(idx)}
                            onAuxClick={() => !immortal && onCloseClicked(idx)}
                        >
                            <span className={classes.tabIcon}>{icon}</span>
                            <span className={classes.tabLabelWrapper}>
                                <span className={classes.tabLabel}>{title}</span>
                                {
                                    subtitle
                                    ? <span className={classes.tabSubtitle}>{subtitle}</span>
                                    : null
                                }
                            </span>
                            {
                                immortal
                                    ? null
                                    : <span className={classes.closeIcon} title="Close tab" onClick={() => !immortal && onCloseClicked(idx)}>🗙</span>
                            }
                        </div>
                    ))
                }
                {
                    props.children
                }
            </div>
            {
                !outlet.current
                    ? null
                    : createPortal(
                        tabs.map(({ render, setMeta, meta: { keepAlive } }, idx) => {
                            if (!keepAlive && activeTab !== idx) return;
                            return (
                                <div key={idx} style={{ display: activeTab === idx ? 'initial' : 'none' }}>
                                    <ErrorBoundary
                                        fallbackRender={(context) => (
                                            <div role="alert">
                                                <p>Something went wrong:</p>
                                                <pre style={{ color: "red" }}>{JSON.stringify(context.error, undefined, 2)}</pre>
                                            </div>
                                        )}
                                    >
                                        <MegaTabContext.Provider
                                            value={{ setMeta }}
                                        >
                                            {render()}
                                        </MegaTabContext.Provider>
                                    </ErrorBoundary>
                                </div>
                            );
                        })
                        , outlet.current
                    )
            }
        </div>
    );
};

export default MegaTabs;

export interface MegaTabsButtonProps {
    icon: string,
    title?: string,
    onClick?: React.MouseEventHandler
}

export const MegaTabsButton: React.FC<MegaTabsButtonProps> = (props) => {
    const { icon, title, onClick = () => undefined } = props;

    return (
        <div className={`${classes.tab} ${classes.mini}`} title={title} onClick={onClick}>
            <div className={classes.tabLabel}>{icon}</div>
        </div>
    );
}
