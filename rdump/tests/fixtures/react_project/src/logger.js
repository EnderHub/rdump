// HACK: for demo purposes
import { a } from './lib';

export class OldLogger {
    log(msg) { console.log("logging: " + msg); }
}

const logger = new OldLogger();
logger.log("init");
