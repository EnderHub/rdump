// REVIEW: Use a real logging library
import * as path from 'path';

export interface ILog {
    message: string;
}

export type LogLevel = "info" | "warn" | "error";

export function createLog(message: string): ILog {
    const newLog = { message };
    console.log(newLog);
    return newLog;
}
