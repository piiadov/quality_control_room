import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    const provider = new TaskTreeDataProvider();
    const treeView = vscode.window.createTreeView('task-run', { treeDataProvider: provider });

    context.subscriptions.push(
        treeView,
        vscode.commands.registerCommand('tasksLauncher.runTask', async (item: TaskTreeItem) => {
            if (item && item.task) {
                await vscode.tasks.executeTask(item.task);
            }
        })
    );
}

export function deactivate() {}

class TaskTreeDataProvider implements vscode.TreeDataProvider<TaskTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<TaskTreeItem | undefined | void> = new vscode.EventEmitter<TaskTreeItem | undefined | void>();
    readonly onDidChangeTreeData: vscode.Event<TaskTreeItem | undefined | void> = this._onDidChangeTreeData.event;

    async getChildren(): Promise<TaskTreeItem[]> {
        // Only show tasks of our custom type
        const tasks = await vscode.tasks.fetchTasks();
        return tasks.map(task => new TaskTreeItem(task));
    }

    getTreeItem(element: TaskTreeItem): vscode.TreeItem {
        return element;
    }
}

class TaskTreeItem extends vscode.TreeItem {
    constructor(public readonly task: vscode.Task) {
        super(task.name, vscode.TreeItemCollapsibleState.None);

        // Now we can safely access the icon property
        let iconId = 'play';
        if (
            task.definition &&
            typeof task.definition.icon === 'object' &&
            typeof task.definition.icon.id === 'string'
        ) {
            iconId = task.definition.icon.id;
        }

        this.iconPath = new vscode.ThemeIcon(iconId);
        this.command = {
            command: 'tasksLauncher.runTask',
            title: 'Run Task',
            arguments: [this]
        };
    }
}
