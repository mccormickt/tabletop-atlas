/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, you can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright Oxide Computer Company
 */

import { camelToSnake, processResponseBody, snakeify, isNotNull } from './util';

/** Success responses from the API */
export type ApiSuccess<Data> = {
	type: 'success';
	response: Response;
	data: Data;
};

// HACK: this has to match what comes from the API in the `Error` schema. We put
// our own copy here so we can test this file statically without generating
// anything
export type ErrorBody = {
	errorCode?: string | null;
	message: string;
	requestId: string;
};

export type ErrorResult =
	// 4xx and 5xx responses from the API
	| {
			type: 'error';
			response: Response;
			data: ErrorBody;
	  }
	// JSON parsing or processing errors within the client. Includes raised Error
	// and response body as a string for debugging.
	| {
			type: 'client_error';
			response: Response;
			error: Error;
			text: string;
	  };

export type ApiResult<Data> = ApiSuccess<Data> | ErrorResult;

/**
 * Convert `Date` to ISO string. Leave other values alone. Used for both request
 * body and query params.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function replacer(_key: string, value: any) {
	if (value instanceof Date) {
		return value.toISOString();
	}
	return value;
}

function encodeQueryParam(key: string, value: unknown) {
	return `${encodeURIComponent(camelToSnake(key))}=${encodeURIComponent(replacer(key, value))}`;
}

export async function handleResponse<Data>(response: Response): Promise<ApiResult<Data>> {
	const respText = await response.text();

	// catch JSON parse or processing errors
	let respJson;
	try {
		// don't bother trying to parse empty responses like 204s
		// TODO: is empty object what we want here?
		respJson = respText.length > 0 ? processResponseBody(JSON.parse(respText)) : {};
	} catch (e) {
		return {
			type: 'client_error',
			response,
			error: e as Error,
			text: respText
		};
	}

	if (!response.ok) {
		return {
			type: 'error',
			response,
			data: respJson as ErrorBody
		};
	}

	// don't validate respJson, just assume it matches the type
	return {
		type: 'success',
		response,
		data: respJson as Data
	};
}

// has to be any. the particular query params types don't like unknown
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type QueryParams = Record<string, any>;

/**
 * Params that get passed to `fetch`. This ends up as an optional second
 * argument to each generated request method. Properties are a subset of
 * `RequestInit`.
 */
export interface FetchParams extends Omit<RequestInit, 'body' | 'method'> {}

/** All arguments to `request()` */
export interface FullParams extends FetchParams {
	path: string;
	query?: QueryParams;
	body?: unknown;
	host?: string;
	method?: string;
}

export interface ApiConfig {
	/**
	 * No host means requests will be sent to the current host. This is used in
	 * the web console.
	 */
	host?: string;
	token?: string;
	baseParams?: FetchParams;
}

export class HttpClient {
	host: string;
	token?: string;
	baseParams: FetchParams;

	constructor({ host = '', baseParams = {}, token }: ApiConfig = {}) {
		this.host = host;
		this.token = token;

		const headers = new Headers({ 'Content-Type': 'application/json' });
		if (token) {
			headers.append('Authorization', `Bearer ${token}`);
		}
		this.baseParams = mergeParams({ headers }, baseParams);
	}

	public async request<Data>({
		body,
		path,
		query,
		host,
		...fetchParams
	}: FullParams): Promise<ApiResult<Data>> {
		const url = (host || this.host) + path + toQueryString(query);

		// Handle binary uploads (File, Blob, etc.) differently from JSON
		let processedBody: string | File | Blob | FormData | undefined;
		let mergedParams = mergeParams(this.baseParams, fetchParams);

		if (body instanceof File || body instanceof Blob || body instanceof FormData) {
			processedBody = body;
			// Remove Content-Type header to let browser set it with boundary for FormData
			// or set appropriate content type for File/Blob
			const headers = new Headers(mergedParams.headers);
			if (body instanceof File || body instanceof Blob) {
				headers.set('Content-Type', 'application/octet-stream');
			} else {
				headers.delete('Content-Type');
			}
			mergedParams = { ...mergedParams, headers };
		} else if (body !== undefined) {
			processedBody = JSON.stringify(snakeify(body), replacer);
		}

		const init = {
			...mergedParams,
			body: processedBody
		};
		return handleResponse(await fetch(url, init));
	}
}

export function mergeParams(a: FetchParams, b: FetchParams): FetchParams {
	// calling `new Headers()` normalizes `HeadersInit`, which could be a Headers
	// object, a plain object, or an array of tuples
	const headers = new Headers(a.headers);
	for (const [key, value] of new Headers(b.headers).entries()) {
		headers.set(key, value);
	}
	return { ...a, ...b, headers };
}

/** Query params with null values filtered out. `"?"` included. */
export function toQueryString(rawQuery?: QueryParams): string {
	const qs = Object.entries(rawQuery || {})
		.filter(([_key, value]) => isNotNull(value))
		.map(([key, value]) =>
			Array.isArray(value)
				? value.map((item) => encodeQueryParam(key, item)).join('&')
				: encodeQueryParam(key, value)
		)
		.join('&');
	return qs ? '?' + qs : '';
}
