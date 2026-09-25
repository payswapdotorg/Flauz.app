// WEB-001 — the protocol type generator.
//
// Reads src/protocol/schema.json (the captured app-server schema export)
// and emits src/protocol/generated.ts — the ONLY protocol types in web/.
// Hand-written duplicates of protocol types are forbidden (Wave-6 kernel
// addendum §3: drift = contract violation).
//
// The generator implements a compact, honest JSON-Schema → TypeScript
// transform: every schema construct it recognizes becomes a precise TS
// type; anything it does not recognize becomes `unknown` (never a
// silently-wrong type). Supported: object properties/required,
// additionalProperties records, arrays, string/number/integer/boolean/
// null, enum/const, oneOf/anyOf unions, allOf intersections, $ref to
// sibling definitions, and the gateway-neutral envelope section.

import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import process from "node:process";

const SCHEMA_PATH = resolve(import.meta.dirname, "../src/protocol/schema.json");
const OUTPUT_PATH = resolve(import.meta.dirname, "../src/protocol/generated.ts");

function fail(message) {
  console.error(`protocol:generate: ${message}`);
  process.exit(1);
}

function pascal(name) {
  return name
    .split(/[^a-zA-Z0-9]+/)
    .filter((part) => part !== "")
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join("");
}

function propertyKey(name) {
  return /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(name) ? name : JSON.stringify(name);
}

function renderSchema(schema, definitions, inlined = new Set()) {
  if (schema === true || schema === undefined) {
    return "unknown";
  }
  if (schema === false) {
    return "never";
  }
  if (typeof schema !== "object") {
    return "unknown";
  }
  const keywords = Object.keys(schema).filter(
    (key) => !["description", "title", "default", "$comment", "$schema"].includes(key),
  );
  if (keywords.length === 0) {
    // A bare annotation-only schema means "any value" — rendered as
    // `unknown`, never a silently-narrower record.
    return "unknown";
  }
  if (typeof schema.$ref === "string") {
    const match = /^#\/(?:definitions|\$defs)\/(.+)$/.exec(schema.$ref);
    if (match) {
      return pascal(match[1]);
    }
    return "unknown";
  }
  if (Array.isArray(schema.enum)) {
    return schema.enum.map((value) => JSON.stringify(value)).join(" | ") || "never";
  }
  if ("const" in schema) {
    return JSON.stringify(schema.const);
  }
  const variants = [...(schema.oneOf ?? []), ...(schema.anyOf ?? [])];
  if (variants.length > 0) {
    const rendered = variants.map((variant) => renderSchema(variant, definitions, inlined));
    return [...new Set(rendered)].join(" | ");
  }
  if (Array.isArray(schema.allOf) && schema.allOf.length > 0) {
    return schema.allOf.map((part) => renderSchema(part, definitions, inlined)).join(" & ");
  }
  if (Array.isArray(schema.type)) {
    return schema.type.map((type) => renderSchema({ ...schema, type }, definitions, inlined)).join(" | ");
  }
  switch (schema.type) {
    case "string":
      return "string";
    case "number":
    case "integer":
      return "number";
    case "boolean":
      return "boolean";
    case "null":
      return "null";
    case "array": {
      const items = renderSchema(schema.items ?? true, definitions, inlined);
      return `${items}[]`;
    }
    case "object":
    default: {
      const properties = schema.properties;
      const required = new Set(schema.required ?? []);
      if (properties && Object.keys(properties).length > 0) {
        const lines = [];
        for (const [name, property] of Object.entries(properties)) {
          const optional = required.has(name) ? "" : "?";
          const rendered = renderSchema(property, definitions, inlined);
          const description =
            property && typeof property === "object" && typeof property.description === "string"
              ? `/** ${property.description.replace(/\*\//g, "* /")} */\n  `
              : "";
          lines.push(`  ${description}${propertyKey(name)}${optional}: ${rendered};`);
        }
        const body = lines.join("\n");
        return inlined.has(body) ? "unknown" : `{\n${body}\n}`;
      }
      if (
        schema.additionalProperties &&
        typeof schema.additionalProperties === "object"
      ) {
        const value = renderSchema(schema.additionalProperties, definitions, inlined);
        return `Record<string, ${value}>`;
      }
      return "Record<string, unknown>";
    }
  }
}

function collectNamedTypes(container, definitions, order, seen) {
  for (const [name, schema] of Object.entries(container)) {
    const typeName = pascal(name);
    if (seen.has(typeName)) {
      continue;
    }
    seen.add(typeName);
    order.push([typeName, schema, typeof schema === "object" && schema?.kind === "inline"]);
  }
}

async function main() {
  let document;
  try {
    document = JSON.parse(await readFile(SCHEMA_PATH, "utf8"));
  } catch (error) {
    fail(`could not read/parse src/protocol/schema.json: ${error.message}`);
  }
  const methods = document.methods;
  if (methods === null || typeof methods !== "object" || Array.isArray(methods)) {
    fail("the schema snapshot has no methods map");
  }
  const definitions = document.definitions ?? document.$defs ?? {};
  const namedOrder = [];
  const seen = new Set();
  collectNamedTypes(definitions, {}, namedOrder, seen);

  const requestEntries = [];
  const notificationEntries = [];
  const serverRequestEntries = [];
  const methodIndex = [];
  const namedTypes = new Map();
  const emittedMethodTypes = [];

  for (const [method, descriptor] of Object.entries(methods)) {
    if (descriptor === null || typeof descriptor !== "object") {
      fail(`method ${method} has no descriptor`);
    }
    const kind = descriptor.kind;
    if (!["request", "notification", "serverRequest"].includes(kind)) {
      fail(`method ${method} has an unknown kind ${JSON.stringify(kind)}`);
    }
    methodIndex.push(method);
    const base = pascal(method);
    const slots = [];
    if (descriptor.params !== undefined) {
      const paramsType = renderSchema(descriptor.params, definitions);
      const typeName = `${base}Params`;
      if (paramsType !== "unknown") {
        namedTypes.set(typeName, paramsType);
        emittedMethodTypes.push(typeName);
        slots.push(`params: ${typeName}`);
      } else {
        slots.push("params: unknown");
      }
    } else {
      slots.push("params: undefined");
    }
    if (kind === "request" && descriptor.result !== undefined) {
      const resultType = renderSchema(descriptor.result, definitions);
      const typeName = `${base}Result`;
      if (resultType !== "unknown") {
        namedTypes.set(typeName, resultType);
        emittedMethodTypes.push(typeName);
        slots.push(`result: ${typeName}`);
      } else {
        slots.push("result: unknown");
      }
    }
    if (kind === "serverRequest" && descriptor.response !== undefined) {
      const responseType = renderSchema(descriptor.response, definitions);
      const typeName = `${base}Response`;
      if (responseType !== "unknown") {
        namedTypes.set(typeName, responseType);
        emittedMethodTypes.push(typeName);
        slots.push(`response: ${typeName}`);
      } else {
        slots.push("response: unknown");
      }
    }
    const entry = `  ${JSON.stringify(method)}: { ${slots.join("; ")} };`;
    if (kind === "request") {
      requestEntries.push(entry);
    } else if (kind === "notification") {
      notificationEntries.push(entry);
    } else {
      serverRequestEntries.push(entry);
    }
  }

  const envelope = document.envelope ?? {};
  const envelopeTypes = [];
  const envelopeSections = [
    ["RequestFrame", "request"],
    ["NotificationFrame", "notification"],
    ["ResponseFrame", "response"],
  ];
  for (const [typeName, key] of envelopeSections) {
    const schema = envelope[key];
    if (schema === undefined) {
      fail(`the schema snapshot envelope is missing the ${key} frame shape`);
    }
    envelopeTypes.push(`export type ${typeName} = ${renderSchema(schema, definitions)};`);
  }

  const lines = [];
  lines.push("// GENERATED FILE — DO NOT EDIT.");
  lines.push("// Provenance: src/protocol/schema.json (the captured app-server schema export).");
  lines.push("// Regenerate with: npm run protocol:export && npm run protocol:generate");
  lines.push("// Hand-written protocol types in web/ are FORBIDDEN (Wave-6 kernel addendum §3).");
  lines.push("");
  lines.push(`export const APP_SERVER_SCHEMA_VERSION = ${Number(document.v ?? 1)};`);
  lines.push(`export const APP_SERVER_SCHEMA_SOURCE = ${JSON.stringify(document.source ?? "unknown")};`);
  lines.push("");
  lines.push(...envelopeTypes);
  lines.push("");
  for (const [typeName, schema] of namedOrder) {
    if (schema === null || typeof schema !== "object") {
      continue;
    }
    lines.push(`export type ${typeName} = ${renderSchema(schema, definitions)};`);
    lines.push("");
  }
  for (const typeName of emittedMethodTypes) {
    lines.push(`export type ${typeName} = ${namedTypes.get(typeName)};`);
    lines.push("");
  }
  lines.push(`export const REQUEST_METHODS = [`);
  for (const method of methodIndex) {
    if (methods[method].kind === "request") {
      lines.push(`  ${JSON.stringify(method)},`);
    }
  }
  lines.push(`] as const;`);
  lines.push("");
  lines.push(`export const NOTIFICATION_METHODS = [`);
  for (const method of methodIndex) {
    if (methods[method].kind === "notification") {
      lines.push(`  ${JSON.stringify(method)},`);
    }
  }
  lines.push(`] as const;`);
  lines.push("");
  lines.push(`export const SERVER_REQUEST_METHODS = [`);
  for (const method of methodIndex) {
    if (methods[method].kind === "serverRequest") {
      lines.push(`  ${JSON.stringify(method)},`);
    }
  }
  lines.push(`] as const;`);
  lines.push("");
  lines.push("/** The per-method protocol surface: params/result shapes by method name. */");
  lines.push("export interface ProtocolMethods {");
  lines.push(...requestEntries);
  lines.push("}");
  lines.push("");
  lines.push("/** Notification params shapes by method name. */");
  lines.push("export interface ProtocolNotifications {");
  lines.push(...notificationEntries);
  lines.push("}");
  lines.push("");
  lines.push("/** Server-initiated request shapes by method name. */");
  lines.push("export interface ProtocolServerRequests {");
  lines.push(...serverRequestEntries);
  lines.push("}");
  lines.push("");
  lines.push("export type ProtocolMethodName = keyof ProtocolMethods;");
  lines.push("export type ProtocolNotificationName = keyof ProtocolNotifications;");
  lines.push("export type ProtocolServerRequestName = keyof ProtocolServerRequests;");
  lines.push("");
  await writeFile(OUTPUT_PATH, `${lines.join("\n")}\n`, "utf8");
  console.log(
    `protocol:generate: emitted ${emittedMethodTypes.length} method type sets, ` +
      `${namedOrder.length} named types, and ${methodIndex.length} methods to src/protocol/generated.ts`,
  );
}

main().catch((error) => fail(error instanceof Error ? error.message : String(error)));
