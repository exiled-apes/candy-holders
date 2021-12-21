"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __asyncValues = (this && this.__asyncValues) || function (o) {
    if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
    var m = o[Symbol.asyncIterator], i;
    return m ? m.call(o) : (o = typeof __values === "function" ? __values(o) : o[Symbol.iterator](), i = {}, verb("next"), verb("throw"), verb("return"), i[Symbol.asyncIterator] = function () { return this; }, i);
    function verb(n) { i[n] = o[n] && function (v) { return new Promise(function (resolve, reject) { v = o[n](v), settle(resolve, reject, v.done, v.value); }); }; }
    function settle(resolve, reject, d, v) { Promise.resolve(v).then(function(v) { resolve({ value: v, done: d }); }, reject); }
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
var web3_js_1 = require("@solana/web3.js");
var readline_1 = require("readline");
var commander_1 = require("commander");
var p_retry_1 = __importDefault(require("p-retry"));
commander_1.program
    .version('0.0.1')
    .option('-e, --rpc-host <string>', 'rpc host', 'https://api.mainnet-beta.solana.com')
    .option('-c, --chill <number>', 'sleep per token (please be nice to free rpc servers) ', '100')
    .parse();
var _a = commander_1.program.opts(), rpcHost = _a.rpcHost, chill = _a.chill;
var connection = new web3_js_1.Connection(rpcHost, 'singleGossip');
function sleep(millis) {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            return [2 /*return*/, new Promise(function (resolve) { return setTimeout(resolve, millis); })];
        });
    });
}
function mineCurrentHolder(tokenAccount) {
    var _a, _b, _c;
    return __awaiter(this, void 0, void 0, function () {
        var largestAccounts, largestPDA, largestWallet, data;
        return __generator(this, function (_d) {
            switch (_d.label) {
                case 0: return [4 /*yield*/, connection.getTokenLargestAccounts(new web3_js_1.PublicKey(tokenAccount))];
                case 1:
                    largestAccounts = _d.sent();
                    largestPDA = largestAccounts.value.shift();
                    return [4 /*yield*/, connection.getParsedAccountInfo(largestPDA === null || largestPDA === void 0 ? void 0 : largestPDA.address)];
                case 2:
                    largestWallet = _d.sent();
                    data = (_a = largestWallet.value) === null || _a === void 0 ? void 0 : _a.data.valueOf();
                    //@ts-ignore
                    return [2 /*return*/, (_c = (_b = data === null || data === void 0 ? void 0 : data.parsed) === null || _b === void 0 ? void 0 : _b.info) === null || _c === void 0 ? void 0 : _c.owner];
            }
        });
    });
}
function main() {
    var e_1, _a;
    return __awaiter(this, void 0, void 0, function () {
        var rest, lineReader, lineReader_1, lineReader_1_1, line, tokenAccount, currentHolder, e_1_1;
        var _this = this;
        return __generator(this, function (_b) {
            switch (_b.label) {
                case 0:
                    rest = parseInt(chill, 10);
                    lineReader = (0, readline_1.createInterface)({
                        input: process.stdin,
                        crlfDelay: Infinity
                    });
                    _b.label = 1;
                case 1:
                    _b.trys.push([1, 8, 9, 14]);
                    lineReader_1 = __asyncValues(lineReader);
                    _b.label = 2;
                case 2: return [4 /*yield*/, lineReader_1.next()];
                case 3:
                    if (!(lineReader_1_1 = _b.sent(), !lineReader_1_1.done)) return [3 /*break*/, 7];
                    line = lineReader_1_1.value;
                    tokenAccount = line.split(' ').pop();
                    return [4 /*yield*/, (0, p_retry_1.default)(function () { return __awaiter(_this, void 0, void 0, function () { return __generator(this, function (_a) {
                            switch (_a.label) {
                                case 0: return [4 /*yield*/, mineCurrentHolder(tokenAccount)];
                                case 1: return [2 /*return*/, _a.sent()];
                            }
                        }); }); }, {
                            onFailedAttempt: function (err) { return console.error("mining ".concat(tokenAccount, " failed."), err); },
                            retries: 4,
                        })];
                case 4:
                    currentHolder = _b.sent();
                    console.log("".concat(currentHolder, ", ").concat(tokenAccount));
                    return [4 /*yield*/, sleep(rest)];
                case 5:
                    _b.sent();
                    _b.label = 6;
                case 6: return [3 /*break*/, 2];
                case 7: return [3 /*break*/, 14];
                case 8:
                    e_1_1 = _b.sent();
                    e_1 = { error: e_1_1 };
                    return [3 /*break*/, 14];
                case 9:
                    _b.trys.push([9, , 12, 13]);
                    if (!(lineReader_1_1 && !lineReader_1_1.done && (_a = lineReader_1.return))) return [3 /*break*/, 11];
                    return [4 /*yield*/, _a.call(lineReader_1)];
                case 10:
                    _b.sent();
                    _b.label = 11;
                case 11: return [3 /*break*/, 13];
                case 12:
                    if (e_1) throw e_1.error;
                    return [7 /*endfinally*/];
                case 13: return [7 /*endfinally*/];
                case 14: return [2 /*return*/];
            }
        });
    });
}
(function () { return __awaiter(void 0, void 0, void 0, function () { return __generator(this, function (_a) {
    switch (_a.label) {
        case 0: return [4 /*yield*/, main()];
        case 1: return [2 /*return*/, _a.sent()];
    }
}); }); })();
