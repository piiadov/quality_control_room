"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
function activate(context) {
    const provider = new TaskTreeDataProvider();
    const treeView = vscode.window.createTreeView('task-run', { treeDataProvider: provider });
    context.subscriptions.push(treeView, vscode.commands.registerCommand('tasksLauncher.runTask', async (item) => {
        if (item && item.task) {
            await vscode.tasks.executeTask(item.task);
        }
    }));
}
function deactivate() { }
class TaskTreeDataProvider {
    _onDidChangeTreeData = new vscode.EventEmitter();
    onDidChangeTreeData = this._onDidChangeTreeData.event;
    async getChildren() {
        // Only show tasks of our custom type
        const tasks = await vscode.tasks.fetchTasks();
        return tasks.map(task => new TaskTreeItem(task));
    }
    getTreeItem(element) {
        return element;
    }
}
class TaskTreeItem extends vscode.TreeItem {
    task;
    constructor(task) {
        super(task.name, vscode.TreeItemCollapsibleState.None);
        this.task = task;
        // Now we can safely access the icon property
        let iconId = 'play';
        if (task.definition &&
            typeof task.definition.icon === 'object' &&
            typeof task.definition.icon.id === 'string') {
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
//# sourceMappingURL=extension.js.map